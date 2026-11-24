use std::fmt;

use crate::models::ResponseStatus;

#[derive(Debug)]
pub enum HikvisionError {
    Http(reqwest::Error),
    Digest(String),
    Xml(quick_xml::DeError),
    /// The camera rejected the request: a non-2xx status, or a 2xx with a
    /// `ResponseStatus` that reports a failure.
    Api {
        status: u16,
        /// Parsed `<ResponseStatus>` when the body contained one. Boxed to
        /// keep `HikvisionError` (and every `Result`) small.
        response: Option<Box<ResponseStatus>>,
        /// The raw response body.
        message: String,
    },
    /// The camera answered successfully but with an unexpected payload,
    /// e.g. an XML `ResponseStatus` where a JPEG image was expected.
    UnexpectedContentType {
        content_type: Option<String>,
        message: String,
    },
}

impl HikvisionError {
    /// Builds an `Api` error from an HTTP status and body.
    pub(crate) fn api(status: u16, message: String) -> Self {
        Self::Api {
            status,
            response: ResponseStatus::parse(&message).map(Box::new),
            message,
        }
    }

    /// The ISAPI `subStatusCode` (e.g. `notSupport`), when available.
    pub fn sub_status_code(&self) -> Option<&str> {
        match self {
            Self::Api {
                response: Some(response),
                ..
            } => response.sub_status_code.as_deref(),
            _ => None,
        }
    }
}

impl fmt::Display for HikvisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(err) => write!(f, "HTTP error: {}", err),
            Self::Digest(err) => write!(f, "Digest authentication error: {}", err),
            Self::Xml(err) => write!(f, "XML error: {}", err),
            Self::Api {
                status,
                response: Some(response),
                ..
            } => write!(f, "ISAPI error ({}): {}", status, response),
            Self::Api {
                status, message, ..
            } => write!(f, "ISAPI error ({}): {}", status, message),
            Self::UnexpectedContentType {
                content_type,
                message,
            } => write!(
                f,
                "Unexpected content type ({}): {}",
                content_type.as_deref().unwrap_or("none"),
                message
            ),
        }
    }
}

impl std::error::Error for HikvisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(err) => Some(err),
            Self::Xml(err) => Some(err),
            Self::Digest(_) | Self::Api { .. } | Self::UnexpectedContentType { .. } => None,
        }
    }
}

impl From<reqwest::Error> for HikvisionError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

impl From<quick_xml::DeError> for HikvisionError {
    fn from(err: quick_xml::DeError) -> Self {
        Self::Xml(err)
    }
}

pub type Result<T> = std::result::Result<T, HikvisionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_stays_small() {
        // Guards against clippy::result_large_err (threshold: 128 bytes).
        assert!(std::mem::size_of::<HikvisionError>() <= 64);
    }
}
