use hikvision_rs::DeviceInfo;

#[test]
fn test_device_info_deserialization() {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <DeviceInfo>
            <deviceName>Front Camera</deviceName>
            <deviceID>DS-2CD2043G2</deviceID>
            <deviceDescription>Network Camera</deviceDescription>
            <deviceLocation>Building A</deviceLocation>
            <model>DS-2CD2043G2-I</model>
            <serialNumber>DS123456789</serialNumber>
            <macAddress>00:11:22:33:44:55</macAddress>
            <firmwareVersion>V5.7.1</firmwareVersion>
            <firmwareReleasedDate>2025-01-01</firmwareReleasedDate>
            <hardwareVersion>1.0</hardwareVersion>
            <deviceType>IPCamera</deviceType>
        </DeviceInfo>
    "#;

    let result: DeviceInfo =
        quick_xml::de::from_str(xml).expect("failed to deserialize DeviceInfo");

    assert_eq!(result.device_name.as_deref(), Some("Front Camera"));
    assert_eq!(result.device_id.as_deref(), Some("DS-2CD2043G2"));
    assert_eq!(result.model.as_deref(), Some("DS-2CD2043G2-I"));
    assert_eq!(result.serial_number.as_deref(), Some("DS123456789"));
    assert_eq!(result.firmware_version.as_deref(), Some("V5.7.1"));
}

#[test]
fn test_device_info_optional_fields() {
    let xml = r#"
        <DeviceInfo>
            <deviceName>Front Camera</deviceName>
            <model>DS-2CD2043G2-I</model>
        </DeviceInfo>
    "#;

    let result: DeviceInfo =
        quick_xml::de::from_str(xml).expect("failed to deserialize DeviceInfo");

    assert_eq!(result.device_name.as_deref(), Some("Front Camera"));
    assert_eq!(result.model.as_deref(), Some("DS-2CD2043G2-I"));
    assert_eq!(result.serial_number, None);
    assert_eq!(result.firmware_version, None);
}
