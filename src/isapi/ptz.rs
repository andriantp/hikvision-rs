use super::IsapiRequest;
use crate::error::Result;
use crate::models::{Preset, PresetList};

pub(crate) fn presets_request(channel: u32) -> IsapiRequest {
    IsapiRequest::get(format!("/ISAPI/PTZCtrl/channels/{}/presets", channel))
}

pub(crate) fn parse_presets(xml: &str) -> Result<Vec<Preset>> {
    let list: PresetList = quick_xml::de::from_str(xml)?;

    Ok(list.presets)
}

pub(crate) fn goto_preset_request(channel: u32, id: u32) -> IsapiRequest {
    IsapiRequest::put(
        format!("/ISAPI/PTZCtrl/channels/{}/presets/{}/goto", channel, id),
        goto_preset_body(id),
    )
}

fn goto_preset_body(id: u32) -> String {
    format!(
        r#"<PTZPreset version="2.0" xmlns="http://www.hikvision.com/ver20/XMLSchema">
    <id>{}</id>
</PTZPreset>"#,
        id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_preset_request() {
        let request = goto_preset_request(1, 2);

        assert_eq!(request.method, reqwest::Method::PUT);
        assert_eq!(request.path, "/ISAPI/PTZCtrl/channels/1/presets/2/goto");

        let body = request.body.expect("body");
        assert!(body.contains("<id>2</id>"));
        assert!(body.starts_with("<PTZPreset"));
        assert!(body.contains("version=\"2.0\""));
        assert!(body.contains("xmlns=\"http://www.hikvision.com/ver20/XMLSchema\""));
    }
}
