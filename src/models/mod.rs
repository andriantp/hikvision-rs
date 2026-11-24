use std::fmt;

/// Connection details of one camera / NVR.
#[derive(Clone)]
pub struct Camera {
    /// Base URL, e.g. `http://192.168.1.64` or `https://cam.local:8443`.
    /// `http://` is assumed when no scheme is given.
    pub host: String,
    pub username: String,
    pub password: String,
}

impl Camera {
    pub fn new(
        host: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            username: username.into(),
            password: password.into(),
        }
    }
}

impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera")
            .field("host", &self.host)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

pub mod device_info;
pub use device_info::DeviceInfo;

pub mod preset;
pub use preset::{Position, Preset, PresetList};

pub mod response_status;
pub use response_status::ResponseStatus;

pub mod snapshot;
pub use snapshot::{Snapshot, SnapshotOptions};
