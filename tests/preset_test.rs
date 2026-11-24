use hikvision_rs::PresetList;

#[test]
fn test_parse_single_preset() {
    let xml = r#"
        <PTZPresetList>
            <PTZPreset>
                <enabled>true</enabled>
                <id>2</id>
                <presetName>command-bridge</presetName>
                <AbsoluteHigh>
                    <elevation>0</elevation>
                    <azimuth>2214</azimuth>
                    <absoluteZoom>10</absoluteZoom>
                </AbsoluteHigh>
            </PTZPreset>
        </PTZPresetList>
    "#;

    let result: PresetList = quick_xml::de::from_str(xml).expect("Failed to parse preset XML");

    assert_eq!(result.presets.len(), 1);

    let preset = &result.presets[0];

    assert!(preset.enabled);
    assert_eq!(preset.id, 2);
    assert_eq!(preset.name, "command-bridge");
    let position = preset
        .position
        .as_ref()
        .expect("position should be present");
    assert_eq!(position.elevation, 0);
    assert_eq!(position.azimuth, 2214);
    assert_eq!(position.zoom, 10);
}

#[test]
fn test_parse_multiple_presets() {
    let xml = r#"
        <PTZPresetList>
            <PTZPreset>
                <enabled>true</enabled>
                <id>2</id>
                <presetName>command-bridge</presetName>
                <AbsoluteHigh>
                    <elevation>0</elevation>
                    <azimuth>2214</azimuth>
                    <absoluteZoom>10</absoluteZoom>
                </AbsoluteHigh>
            </PTZPreset>
            <PTZPreset>
                <enabled>true</enabled>
                <id>3</id>
                <presetName>command-page</presetName>
                <AbsoluteHigh>
                    <elevation>0</elevation>
                    <azimuth>1701</azimuth>
                    <absoluteZoom>10</absoluteZoom>
                </AbsoluteHigh>
            </PTZPreset>
        </PTZPresetList>
    "#;

    let result: PresetList = quick_xml::de::from_str(xml).expect("Failed to parse preset XML");

    assert_eq!(result.presets.len(), 2);
    assert_eq!(result.presets[0].name, "command-bridge");
    assert_eq!(result.presets[1].name, "command-page");
}

#[test]
fn test_parse_preset_without_position_and_name() {
    let xml = r#"
        <PTZPresetList version="2.0" xmlns="http://www.hikvision.com/ver20/XMLSchema">
            <PTZPreset>
                <id>1</id>
            </PTZPreset>
            <PTZPreset>
                <enabled>true</enabled>
                <id>2</id>
                <presetName>gate</presetName>
                <AbsoluteHigh>
                    <elevation>5</elevation>
                    <azimuth>100</azimuth>
                    <absoluteZoom>10</absoluteZoom>
                </AbsoluteHigh>
            </PTZPreset>
        </PTZPresetList>
    "#;

    let result: PresetList = quick_xml::de::from_str(xml).expect("Failed to parse preset XML");

    assert_eq!(result.presets.len(), 2);

    let first = &result.presets[0];
    assert_eq!(first.id, 1);
    assert!(!first.enabled);
    assert_eq!(first.name, "");
    assert!(first.position.is_none());

    assert_eq!(result.presets[1].name, "gate");
    assert!(result.presets[1].position.is_some());
}

#[test]
fn test_parse_empty_preset_list() {
    let xml = r#"<PTZPresetList></PTZPresetList>"#;

    let result: PresetList = quick_xml::de::from_str(xml).expect("Failed to parse preset XML");

    assert!(result.presets.is_empty());
}
