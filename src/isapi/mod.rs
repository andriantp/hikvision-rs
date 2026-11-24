//! ISAPI endpoint definitions and response parsing, shared by the blocking
//! and async clients. Nothing here does I/O.

pub(crate) mod ptz;
pub(crate) mod snapshot;
pub(crate) mod system;

use reqwest::Method;

use crate::error::{HikvisionError, Result};
use crate::models::ResponseStatus;

/// One ISAPI call: what to send.
pub(crate) struct IsapiRequest {
    pub method: Method,
    pub path: String,
    pub query: Vec<(&'static str, String)>,
    pub body: Option<String>,
}

impl IsapiRequest {
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: Method::GET,
            path: path.into(),
            query: Vec::new(),
            body: None,
        }
    }

    pub fn put(path: impl Into<String>, body: String) -> Self {
        Self {
            method: Method::PUT,
            path: path.into(),
            query: Vec::new(),
            body: Some(body),
        }
    }
}

/// Joins the camera host and an ISAPI path.
///
/// Trailing slashes on the host are removed so `http://cam/` + `/ISAPI/...`
/// does not become `http://cam//ISAPI/...`, and `http://` is assumed when the
/// host has no scheme (e.g. `192.168.1.10`).
pub(crate) fn build_url(host: &str, path: &str) -> String {
    let host = host.trim().trim_end_matches('/');
    let path = path.trim_start_matches('/');

    if host.contains("://") {
        format!("{}/{}", host, path)
    } else {
        format!("http://{}/{}", host, path)
    }
}

/// Checks the `ResponseStatus` body of a successful (2xx) write. Some firmware
/// reports failures this way instead of with an HTTP error status.
pub(crate) fn check_response_status(status: u16, body: String) -> Result<()> {
    match ResponseStatus::parse(&body) {
        Some(response) if !response.is_success() => Err(HikvisionError::Api {
            status,
            response: Some(Box::new(response)),
            message: body,
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_plain() {
        assert_eq!(
            build_url("http://cam:80", "/ISAPI/System/deviceInfo"),
            "http://cam:80/ISAPI/System/deviceInfo"
        );
    }

    #[test]
    fn test_build_url_trailing_slash() {
        assert_eq!(
            build_url("http://cam/", "/ISAPI/System/deviceInfo"),
            "http://cam/ISAPI/System/deviceInfo"
        );
    }

    #[test]
    fn test_build_url_without_scheme() {
        assert_eq!(
            build_url("192.168.1.10", "/ISAPI/System/deviceInfo"),
            "http://192.168.1.10/ISAPI/System/deviceInfo"
        );
    }

    #[test]
    fn test_build_url_keeps_https() {
        assert_eq!(
            build_url("https://cam", "/ISAPI/System/deviceInfo"),
            "https://cam/ISAPI/System/deviceInfo"
        );
    }

    #[test]
    fn test_check_response_status() {
        let ok = r#"<ResponseStatus><statusCode>1</statusCode><statusString>OK</statusString></ResponseStatus>"#;
        assert!(check_response_status(200, ok.to_string()).is_ok());

        assert!(check_response_status(200, String::new()).is_ok());

        let failed = r#"<ResponseStatus><statusCode>3</statusCode><statusString>Device Error</statusString><subStatusCode>deviceError</subStatusCode></ResponseStatus>"#;
        let err = check_response_status(200, failed.to_string()).unwrap_err();
        assert_eq!(err.sub_status_code(), Some("deviceError"));
    }
}
