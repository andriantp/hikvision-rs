use hikvision_rs::SnapshotOptions;

#[test]
fn test_default_snapshot_options() {
    let options = SnapshotOptions::default();

    assert_eq!(options.width, None);
    assert_eq!(options.height, None);
    assert_eq!(options.image_quality, "better");
}

#[test]
fn test_custom_snapshot_options() {
    let options = SnapshotOptions {
        width: Some(1920),
        height: Some(1080),
        image_quality: "best".to_string(),
    };

    assert_eq!(options.width, Some(1920));
    assert_eq!(options.height, Some(1080));
    assert_eq!(options.image_quality, "best");
}
