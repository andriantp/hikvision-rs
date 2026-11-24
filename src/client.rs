use std::time::Duration;

use reqwest::blocking::{Client as HttpClient, Request, Response};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Method, StatusCode};

use crate::auth::{self, AuthCache, AuthTarget};
use crate::device::Device;
use crate::error::{HikvisionError, Result};
use crate::isapi::{self, IsapiRequest, ptz, snapshot, system};
use crate::models::{Camera, DeviceInfo, Preset, Snapshot, SnapshotOptions};

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Total timeout per HTTP request. Zero means the default (30 s).
    pub timeout: Duration,

    /// Accept invalid / self-signed TLS certificates, which most cameras use
    /// for HTTPS. This disables certificate verification: only enable it on
    /// networks you trust.
    pub accept_invalid_certs: bool,
}

impl ClientConfig {
    pub(crate) fn normalized(&self) -> Self {
        Self {
            timeout: if self.timeout.is_zero() {
                DEFAULT_TIMEOUT
            } else {
                self.timeout
            },
            accept_invalid_certs: self.accept_invalid_certs,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            accept_invalid_certs: false,
        }
    }
}

/// Blocking ISAPI client.
///
/// Cheap to clone: clones share the connection pool and the digest cache.
/// Do not use it from inside an async runtime (reqwest's blocking client
/// panics there); use [`AsyncClient`](crate::AsyncClient) instead.
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    auth: AuthCache,
}

impl Client {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let config = config.normalized();

        let http = HttpClient::builder()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()?;

        Ok(Self {
            http,
            auth: AuthCache::default(),
        })
    }

    /// Binds this client to one camera, so calls don't need `&Camera`.
    pub fn device(&self, camera: Camera) -> Device {
        Device::new(self.clone(), camera)
    }

    pub fn get_device_info(&self, camera: &Camera) -> Result<DeviceInfo> {
        let xml = self
            .execute(camera, system::device_info_request())?
            .text()?;

        system::parse_device_info(&xml)
    }

    pub fn get_presets(&self, camera: &Camera, channel: u32) -> Result<Vec<Preset>> {
        let xml = self
            .execute(camera, ptz::presets_request(channel))?
            .text()?;

        ptz::parse_presets(&xml)
    }

    pub fn goto_preset(&self, camera: &Camera, channel: u32, id: u32) -> Result<()> {
        let response = self.execute(camera, ptz::goto_preset_request(channel, id))?;
        let status = response.status().as_u16();

        isapi::check_response_status(status, response.text()?)
    }

    pub fn get_snapshot(
        &self,
        camera: &Camera,
        channel: u32,
        options: SnapshotOptions,
    ) -> Result<Snapshot> {
        let response = self.execute(camera, snapshot::snapshot_request(channel, options))?;

        match snapshot::image_content_type(response.headers()) {
            Ok(content_type) => Ok(Snapshot {
                content_type,
                data: response.bytes()?,
            }),
            Err(content_type) => {
                let body = response
                    .text()
                    .unwrap_or_else(|err| format!("Failed to read response body: {}", err));

                Err(snapshot::unexpected_content_type(content_type, body))
            }
        }
    }

    fn execute(&self, camera: &Camera, request: IsapiRequest) -> Result<Response> {
        self.request(
            request.method,
            camera,
            &request.path,
            &request.query,
            request.body.map(String::into_bytes),
        )
    }

    /// Sends a raw ISAPI request with digest authentication.
    ///
    /// Non-2xx responses are returned as [`HikvisionError::Api`].
    pub fn request(
        &self,
        method: Method,
        camera: &Camera,
        path: &str,
        query: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        let mut builder = self
            .http
            .request(method, isapi::build_url(&camera.host, path));

        if !query.is_empty() {
            builder = builder.query(query);
        }

        if let Some(body) = body {
            builder = builder.header(CONTENT_TYPE, "application/xml").body(body);
        }

        let request = builder.build()?;

        let target = AuthTarget::new(request.url(), &camera.username);
        let context = target.context(
            camera,
            request.method(),
            request.body().and_then(|body| body.as_bytes()),
        );

        // Fast path: sign with the cached challenge, a single round-trip.
        let cached = self.auth.cached_answer(&target.key, &context);
        let response = self.send(&request, cached)?;

        if response.status() != StatusCode::UNAUTHORIZED {
            return Self::validate_response(response);
        }

        // No cached challenge, or it expired / was rejected: handshake with
        // the challenge from this 401 and retry once.
        let Some(answer) = self
            .auth
            .answer_challenge(&target.key, response.headers(), &context)?
        else {
            return Self::validate_response(response);
        };

        let response = self.send(&request, Some(answer))?;

        if response.status() == StatusCode::UNAUTHORIZED {
            // Wrong credentials: don't keep a challenge that cannot succeed.
            self.auth.forget(&target.key);
        }

        Self::validate_response(response)
    }

    fn send(&self, request: &Request, authorization: Option<String>) -> Result<Response> {
        // Bodies are always in-memory bytes, so cloning cannot fail in practice.
        let mut request = request
            .try_clone()
            .ok_or_else(|| HikvisionError::Digest("request body is not cloneable".to_string()))?;

        if let Some(authorization) = authorization {
            request
                .headers_mut()
                .insert(AUTHORIZATION, auth::header_value(&authorization)?);
        }

        Ok(self.http.execute(request)?)
    }

    fn validate_response(response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let message = response
            .text()
            .unwrap_or_else(|err| format!("Failed to read error response: {}", err));

        Err(HikvisionError::api(status, message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_timeout_uses_default() {
        let config = ClientConfig {
            timeout: Duration::ZERO,
            ..ClientConfig::default()
        };

        assert_eq!(config.normalized().timeout, DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_custom_config_is_preserved() {
        let config = ClientConfig {
            timeout: Duration::from_secs(60),
            accept_invalid_certs: true,
        }
        .normalized();

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.accept_invalid_certs);
    }
}
