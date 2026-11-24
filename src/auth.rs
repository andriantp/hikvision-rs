//! Digest authentication shared by the blocking and async clients.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use digest_auth::{AuthContext, HttpMethod, WwwAuthenticateHeader};
use reqwest::header::{HeaderMap, HeaderValue, WWW_AUTHENTICATE};
use reqwest::{Method, Url};

use crate::error::{HikvisionError, Result};
use crate::models::Camera;

/// Cached digest challenges are scoped per camera origin and username.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AuthKey {
    origin: String,
    username: String,
}

/// What a request is signed for: cache key plus the digest `uri` value.
pub(crate) struct AuthTarget {
    pub key: AuthKey,
    uri: String,
}

impl AuthTarget {
    pub fn new(url: &Url, username: &str) -> Self {
        Self {
            key: AuthKey {
                origin: url.origin().ascii_serialization(),
                username: username.to_string(),
            },
            uri: request_uri(url),
        }
    }

    pub fn context<'a>(
        &'a self,
        camera: &'a Camera,
        method: &'a Method,
        body: Option<&'a [u8]>,
    ) -> AuthContext<'a> {
        AuthContext::new_with_method(
            camera.username.as_str(),
            camera.password.as_str(),
            self.uri.as_str(),
            body,
            HttpMethod::from(method.as_str()),
        )
    }
}

/// Last digest challenge seen per camera. Reusing it (with an increasing
/// `nc`) avoids the unauthenticated 401 round-trip on every request.
/// Cloning shares the cache.
#[derive(Debug, Clone, Default)]
pub(crate) struct AuthCache {
    challenges: Arc<Mutex<HashMap<AuthKey, WwwAuthenticateHeader>>>,
}

impl AuthCache {
    /// `Authorization` value signed with the cached challenge, if any.
    pub fn cached_answer(&self, key: &AuthKey, context: &AuthContext) -> Option<String> {
        let mut challenges = self.lock();
        let challenge = challenges.get_mut(key)?;

        match challenge.respond(context) {
            Ok(answer) => Some(answer.to_header_string()),
            Err(_) => {
                challenges.remove(key);
                None
            }
        }
    }

    /// Answers the Digest challenge of a 401 response and caches it.
    /// Returns `None` (and forgets the key) when there is no Digest challenge.
    pub fn answer_challenge(
        &self,
        key: &AuthKey,
        headers: &HeaderMap,
        context: &AuthContext,
    ) -> Result<Option<String>> {
        let Some(mut challenge) = digest_challenge(headers)? else {
            self.forget(key);
            return Ok(None);
        };

        let answer = challenge
            .respond(context)
            .map_err(|err| HikvisionError::Digest(err.to_string()))?
            .to_header_string();

        self.lock().insert(key.clone(), challenge);

        Ok(Some(answer))
    }

    pub fn forget(&self, key: &AuthKey) {
        self.lock().remove(key);
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<AuthKey, WwwAuthenticateHeader>> {
        // The cache holds no invariants that a panic could break.
        self.challenges
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(crate) fn header_value(authorization: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(authorization).map_err(|err| HikvisionError::Digest(err.to_string()))
}

/// The digest `uri` value: path plus query, as sent on the request line.
fn request_uri(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{}", url.path(), query),
        None => url.path().to_string(),
    }
}

/// Picks the Digest challenge from the `WWW-Authenticate` headers.
///
/// Hikvision devices may send several challenges (e.g. `Basic` and `Digest`);
/// only the Digest one is usable here.
fn digest_challenge(headers: &HeaderMap) -> Result<Option<WwwAuthenticateHeader>> {
    let Some(value) = headers
        .get_all(WWW_AUTHENTICATE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| {
            value
                .trim_start()
                .get(..6)
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("digest"))
        })
    else {
        return Ok(None);
    };

    digest_auth::parse(value)
        .map(Some)
        .map_err(|err| HikvisionError::Digest(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_uri_includes_query() {
        let url = Url::parse("http://cam/ISAPI/Streaming/channels/1/picture?a=1&b=x%20y").unwrap();
        assert_eq!(
            request_uri(&url),
            "/ISAPI/Streaming/channels/1/picture?a=1&b=x%20y"
        );
    }

    #[test]
    fn test_digest_challenge_skips_basic() {
        let mut headers = HeaderMap::new();
        headers.append(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"cam\""),
        );
        headers.append(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Digest realm=\"cam\", nonce=\"abc\", qop=\"auth\""),
        );

        let challenge = digest_challenge(&headers)
            .unwrap()
            .expect("digest challenge");
        assert_eq!(challenge.nonce, "abc");
    }

    #[test]
    fn test_digest_challenge_none_without_digest() {
        let mut headers = HeaderMap::new();
        headers.append(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"cam\""),
        );

        assert!(digest_challenge(&headers).unwrap().is_none());
    }
}
