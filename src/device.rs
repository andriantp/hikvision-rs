use crate::async_client::AsyncClient;
use crate::client::Client;
use crate::error::Result;
use crate::models::{Camera, DeviceInfo, Preset, Snapshot, SnapshotOptions};

/// A [`Client`] bound to one [`Camera`].
///
/// ```no_run
/// use hikvision_rs::{Camera, Client, ClientConfig};
///
/// let client = Client::new(ClientConfig::default())?;
/// let cam = client.device(Camera::new("http://192.168.1.64", "admin", "password"));
///
/// let info = cam.get_device_info()?;
/// cam.goto_preset(1, 2)?;
/// # Ok::<(), hikvision_rs::HikvisionError>(())
/// ```
#[derive(Clone, Debug)]
pub struct Device {
    client: Client,
    camera: Camera,
}

impl Device {
    pub fn new(client: Client, camera: Camera) -> Self {
        Self { client, camera }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn get_device_info(&self) -> Result<DeviceInfo> {
        self.client.get_device_info(&self.camera)
    }

    pub fn get_presets(&self, channel: u32) -> Result<Vec<Preset>> {
        self.client.get_presets(&self.camera, channel)
    }

    pub fn goto_preset(&self, channel: u32, id: u32) -> Result<()> {
        self.client.goto_preset(&self.camera, channel, id)
    }

    pub fn get_snapshot(&self, channel: u32, options: SnapshotOptions) -> Result<Snapshot> {
        self.client.get_snapshot(&self.camera, channel, options)
    }
}

/// An [`AsyncClient`] bound to one [`Camera`].
#[derive(Clone, Debug)]
pub struct AsyncDevice {
    client: AsyncClient,
    camera: Camera,
}

impl AsyncDevice {
    pub fn new(client: AsyncClient, camera: Camera) -> Self {
        Self { client, camera }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn client(&self) -> &AsyncClient {
        &self.client
    }

    pub async fn get_device_info(&self) -> Result<DeviceInfo> {
        self.client.get_device_info(&self.camera).await
    }

    pub async fn get_presets(&self, channel: u32) -> Result<Vec<Preset>> {
        self.client.get_presets(&self.camera, channel).await
    }

    pub async fn goto_preset(&self, channel: u32, id: u32) -> Result<()> {
        self.client.goto_preset(&self.camera, channel, id).await
    }

    pub async fn get_snapshot(&self, channel: u32, options: SnapshotOptions) -> Result<Snapshot> {
        self.client
            .get_snapshot(&self.camera, channel, options)
            .await
    }
}
