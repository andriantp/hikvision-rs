//! End-to-end tests of the blocking client against a local ISAPI mock.

mod common;

use common::{JPEG, MockCamera, PASSWORD, USERNAME, closed_port};
use hikvision_rs::{Camera, Client, ClientConfig, HikvisionError, SnapshotOptions};

fn client() -> Client {
    Client::new(ClientConfig::default()).unwrap()
}

#[test]
fn test_challenge_is_reused_across_requests() {
    let mock = MockCamera::start(0);
    let camera = mock.camera(PASSWORD);
    let client = client();

    for _ in 0..3 {
        let info = client.get_device_info(&camera).unwrap();
        assert_eq!(info.model.as_deref(), Some("DS-MOCK"));
    }

    let seen = mock.seen();
    // 1 unauthenticated 401 + 3 authenticated requests (not 6 round-trips).
    assert_eq!(seen.len(), 4, "{seen:?}");
    assert!(!seen[0].authorized);
    assert!(seen[1..].iter().all(|request| request.authorized));
    assert_eq!(
        seen[1..]
            .iter()
            .map(|request| request.nc)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3)]
    );
}

#[test]
fn test_expired_nonce_triggers_new_handshake() {
    // The mock rotates its nonce after every 2 successful requests.
    let mock = MockCamera::start(2);
    let camera = mock.camera(PASSWORD);
    let client = client();

    for _ in 0..3 {
        client.get_device_info(&camera).unwrap();
    }

    let seen = mock.seen();
    // 401, ok, ok, (nonce rotated) 401 stale, ok
    assert_eq!(seen.len(), 5, "{seen:?}");
    assert_eq!(
        seen.iter()
            .map(|request| request.authorized)
            .collect::<Vec<_>>(),
        vec![false, true, true, false, true]
    );
}

#[test]
fn test_put_body_is_signed() {
    let mock = MockCamera::start(0);
    let camera = mock.camera(PASSWORD);
    let client = client();

    client.goto_preset(&camera, 1, 2).unwrap();
    client.goto_preset(&camera, 1, 3).unwrap();

    let seen = mock.seen();
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(seen.iter().all(|request| request.method == "PUT"));
    assert!(seen[1].authorized && seen[2].authorized);
}

#[test]
fn test_wrong_password_returns_401_without_looping() {
    let mock = MockCamera::start(0);
    let camera = mock.camera("wrong");
    let client = client();

    for _ in 0..2 {
        let err = client.get_device_info(&camera).unwrap_err();
        assert!(
            matches!(err, HikvisionError::Api { status: 401, .. }),
            "{err:?}"
        );
    }

    // Each call: unauthenticated attempt + one signed retry; nothing cached.
    assert_eq!(mock.seen().len(), 4);
}

#[test]
fn test_connection_error_is_http_error() {
    let camera = Camera::new(
        format!("http://127.0.0.1:{}", closed_port()),
        USERNAME,
        PASSWORD,
    );

    let err = client().get_device_info(&camera).unwrap_err();
    assert!(matches!(err, HikvisionError::Http(_)), "{err:?}");
}

#[test]
fn test_device_binds_camera() {
    let mock = MockCamera::start(0);
    let device = client().device(mock.camera(PASSWORD));

    assert_eq!(
        device.get_device_info().unwrap().model.as_deref(),
        Some("DS-MOCK")
    );
    device.goto_preset(1, 2).unwrap();

    // The device shares the client's digest cache: one handshake only.
    assert_eq!(mock.seen().len(), 3);
}

#[test]
fn test_snapshot_returns_image() {
    let mock = MockCamera::start(0);
    let snapshot = client()
        .get_snapshot(&mock.camera(PASSWORD), 1, SnapshotOptions::default())
        .unwrap();

    assert_eq!(snapshot.content_type, "image/jpeg");
    assert_eq!(&snapshot.data[..], JPEG);

    let seen = mock.seen();
    assert_eq!(
        seen.last().unwrap().path,
        "/ISAPI/Streaming/channels/1/picture"
    );
}

#[test]
fn test_snapshot_xml_answer_is_an_error() {
    let mock = MockCamera::start(0);
    let err = client()
        .get_snapshot(&mock.camera(PASSWORD), 2, SnapshotOptions::default())
        .unwrap_err();

    match err {
        HikvisionError::UnexpectedContentType {
            content_type,
            message,
        } => {
            assert_eq!(content_type.as_deref(), Some("application/xml"));
            assert!(message.contains("notSupport"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn test_error_status_is_parsed() {
    let mock = MockCamera::start(0);
    let err = client()
        .goto_preset(&mock.camera(PASSWORD), 1, 99)
        .unwrap_err();

    assert_eq!(err.sub_status_code(), Some("invalidOperation"));
    match &err {
        HikvisionError::Api {
            status,
            response: Some(response),
            ..
        } => {
            assert_eq!(*status, 403);
            assert_eq!(response.status_code, Some(4));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "ISAPI error (403): Invalid Operation (4), invalidOperation"
    );
}

#[test]
fn test_failure_in_200_body_is_an_error() {
    let mock = MockCamera::start(0);
    let err = client()
        .goto_preset(&mock.camera(PASSWORD), 1, 98)
        .unwrap_err();

    assert!(
        matches!(err, HikvisionError::Api { status: 200, .. }),
        "{err:?}"
    );
    assert_eq!(err.sub_status_code(), Some("deviceError"));
}
