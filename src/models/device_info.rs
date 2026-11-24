use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeviceInfo {
    #[serde(rename = "deviceName")]
    pub device_name: Option<String>,

    #[serde(rename = "deviceID")]
    pub device_id: Option<String>,

    #[serde(rename = "deviceDescription")]
    pub device_description: Option<String>,

    #[serde(rename = "deviceLocation")]
    pub device_location: Option<String>,

    pub model: Option<String>,

    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,

    #[serde(rename = "firmwareVersion")]
    pub firmware_version: Option<String>,
}
