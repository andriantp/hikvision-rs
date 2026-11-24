use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub image_quality: String,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            image_quality: "better".to_string(),
        }
    }
}

/// A captured image. `data` is the response body as received, without copying.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// The response `Content-Type`, e.g. `image/jpeg`.
    pub content_type: String,
    pub data: Bytes,
}
