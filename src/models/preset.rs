use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Preset {
    #[serde(default)]
    pub enabled: bool,

    pub id: u32,

    /// Empty when the camera does not report a name for this preset.
    #[serde(rename = "presetName", default)]
    pub name: String,

    /// `None` when the camera does not report a position (e.g. some NVRs or
    /// presets that were never saved).
    #[serde(rename = "AbsoluteHigh", default)]
    pub position: Option<Position>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Position {
    #[serde(default)]
    pub elevation: i32,

    #[serde(default)]
    pub azimuth: i32,

    #[serde(rename = "absoluteZoom", default)]
    pub zoom: i32,
}

#[derive(Debug, Deserialize)]
pub struct PresetList {
    #[serde(rename = "PTZPreset", default)]
    pub presets: Vec<Preset>,
}
