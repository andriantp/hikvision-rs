use hikvision_rs::ClientConfig;
use std::time::Duration;

#[test]
fn test_default_config() {
    let config = ClientConfig::default();

    assert_eq!(config.timeout, Duration::from_secs(30));
}
