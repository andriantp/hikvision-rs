//! End-to-end tests of the async client against a local ISAPI mock.

mod common;

use common::{JPEG, MockCamera, PASSWORD, USERNAME, closed_port};
use hikvision_rs::{AsyncClient, Camera, ClientConfig, HikvisionError, SnapshotOptions};

fn client() -> AsyncClient {
    AsyncClient::new(ClientConfig::default()).unwrap()
}

#[tokio::test]
async fn test_challenge_is_reused_across_requests() {
    let mock = MockCamera::start(0);
    let camera = mock.camera(PASSWORD);
    let client = client();

    for _ in 0..3 {
        let info = client.get_device_info(&camera).await.unwrap();
        assert_eq!(info.model.as_deref(), Some("DS-MOCK"));
    }

    let seen = mock.seen();
    assert_eq!(seen.len(), 4, "{seen:?}");
    assert_eq!(
        seen[1..]
            .iter()
            .map(|request| request.nc)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3)]
    );
}

#[tokio::test]
async fn test_expired_nonce_triggers_new_handshake() {
    let mock = MockCamera::start(2);
    let camera = mock.camera(PASSWORD);
    let client = client();

    for _ in 0..3 {
        client.get_device_info(&camera).await.unwrap();
    }

    assert_eq!(
        mock.seen()
            .iter()
            .map(|request| request.authorized)
            .collect::<Vec<_>>(),
        vec![false, true, true, false, true]
    );
}

#[tokio::test]
async fn test_device_presets_and_snapshot() {
    let mock = MockCamera::start(0);
    let device = client().device(mock.camera(PASSWORD));

    device.goto_preset(1, 2).await.unwrap();

    let snapshot = device
        .get_snapshot(1, SnapshotOptions::default())
        .await
        .unwrap();
    assert_eq!(&snapshot.data[..], JPEG);

    let err = device.goto_preset(1, 99).await.unwrap_err();
    assert_eq!(err.sub_status_code(), Some("invalidOperation"));

    // 1 handshake + 3 signed requests.
    assert_eq!(mock.seen().len(), 4);
}

#[tokio::test]
async fn test_wrong_password_returns_401() {
    let mock = MockCamera::start(0);
    let err = client()
        .get_device_info(&mock.camera("wrong"))
        .await
        .unwrap_err();

    assert!(
        matches!(err, HikvisionError::Api { status: 401, .. }),
        "{err:?}"
    );
    assert_eq!(mock.seen().len(), 2);
}

#[tokio::test]
async fn test_connection_error_is_http_error() {
    let camera = Camera::new(
        format!("http://127.0.0.1:{}", closed_port()),
        USERNAME,
        PASSWORD,
    );

    let err = client().get_device_info(&camera).await.unwrap_err();
    assert!(matches!(err, HikvisionError::Http(_)), "{err:?}");
}

#[test]
fn test_async_client_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AsyncClient>();
    assert_send_sync::<hikvision_rs::AsyncDevice>();
}
