use reqwest::header::{CONTENT_TYPE, HeaderMap};

use super::IsapiRequest;
use crate::error::HikvisionError;
use crate::models::SnapshotOptions;

pub(crate) fn snapshot_request(channel: u32, options: SnapshotOptions) -> IsapiRequest {
    let mut request = IsapiRequest::get(format!("/ISAPI/Streaming/channels/{}/picture", channel));

    request
        .query
        .push(("snapShotImageType", "JPEG".to_string()));
    request.query.push(("imageQuality", options.image_quality));

    if let Some(width) = options.width {
        request
            .query
            .push(("videoResolutionWidth", width.to_string()));
    }

    if let Some(height) = options.height {
        request
            .query
            .push(("videoResolutionHeight", height.to_string()));
    }

    request
}

/// Returns the image `Content-Type`, or `Err(content_type)` when the response
/// is not an image (some firmware answers 200 with an XML `ResponseStatus`).
pub(crate) fn image_content_type(
    headers: &HeaderMap,
) -> std::result::Result<String, Option<String>> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    match content_type {
        Some(value) if is_image(&value) => Ok(value),
        other => Err(other),
    }
}

pub(crate) fn unexpected_content_type(
    content_type: Option<String>,
    body: String,
) -> HikvisionError {
    HikvisionError::UnexpectedContentType {
        content_type,
        message: body,
    }
}

fn is_image(content_type: &str) -> bool {
    content_type
        .trim()
        .to_ascii_lowercase()
        .starts_with("image/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image() {
        assert!(is_image("image/jpeg"));
        assert!(is_image("Image/JPEG; charset=binary"));
        assert!(!is_image("application/xml"));
        assert!(!is_image("text/html"));
    }

    #[test]
    fn test_snapshot_request_query() {
        let request = snapshot_request(
            101,
            SnapshotOptions {
                width: Some(1920),
                height: None,
                image_quality: "best".to_string(),
            },
        );

        assert_eq!(request.path, "/ISAPI/Streaming/channels/101/picture");
        assert_eq!(
            request.query,
            vec![
                ("snapShotImageType", "JPEG".to_string()),
                ("imageQuality", "best".to_string()),
                ("videoResolutionWidth", "1920".to_string()),
            ]
        );
    }
}
