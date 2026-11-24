//! Minimal local ISAPI mock with digest auth, shared by the integration tests.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use digest_auth::{AuthContext, AuthorizationHeader, HttpMethod};
use hikvision_rs::Camera;

pub const USERNAME: &str = "admin";
pub const PASSWORD: &str = "secret";
const REALM: &str = "IP Camera";

const DEVICE_INFO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<DeviceInfo version="2.0" xmlns="http://www.hikvision.com/ver20/XMLSchema">
    <deviceName>Mock</deviceName>
    <model>DS-MOCK</model>
</DeviceInfo>"#;

#[derive(Debug, Clone)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub authorized: bool,
    pub nc: Option<u32>,
}

#[derive(Default)]
struct State {
    nonce_generation: u32,
    last_nc: u32,
    /// Rotate the nonce after this many successful requests (0 = never).
    rotate_after: u32,
    served_with_nonce: u32,
    seen: Vec<Seen>,
}

impl State {
    fn nonce(&self) -> String {
        format!("nonce{}", self.nonce_generation)
    }
}

pub struct MockCamera {
    pub host: String,
    state: Arc<Mutex<State>>,
}

impl MockCamera {
    pub fn start(rotate_after: u32) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State {
            rotate_after,
            ..State::default()
        }));

        let server_state = Arc::clone(&state);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                handle(stream, &server_state);
            }
        });

        Self { host, state }
    }

    pub fn camera(&self, password: &str) -> Camera {
        Camera {
            host: self.host.clone(),
            username: USERNAME.to_string(),
            password: password.to_string(),
        }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.state.lock().unwrap().seen.clone()
    }
}

fn handle(mut stream: TcpStream, state: &Mutex<State>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let uri = parts.next().unwrap_or_default().to_string();

    let mut authorization = None;
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            match name.to_ascii_lowercase().as_str() {
                "authorization" => authorization = Some(value.trim().to_string()),
                "content-length" => content_length = value.trim().parse().unwrap(),
                _ => {}
            }
        }
    }

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).unwrap();

    let mut state = state.lock().unwrap();

    let parsed = authorization
        .as_deref()
        .and_then(|value| AuthorizationHeader::parse(value).ok());

    let (ok, stale) = match &parsed {
        None => (false, false),
        Some(header) => {
            let stale = header.nonce != state.nonce();
            let valid = !stale
                && header.uri == uri
                && header.nc > state.last_nc
                && verify(header, &method, &body);
            (valid, stale)
        }
    };

    state.seen.push(Seen {
        method: method.clone(),
        path: uri.split('?').next().unwrap_or_default().to_string(),
        authorized: ok,
        nc: parsed.as_ref().map(|header| header.nc),
    });

    let response = if ok {
        state.last_nc = parsed.unwrap().nc;
        state.served_with_nonce += 1;
        if state.rotate_after > 0 && state.served_with_nonce >= state.rotate_after {
            state.nonce_generation += 1;
            state.served_with_nonce = 0;
            state.last_nc = 0;
        }

        route(&method, &uri)
    } else {
        // Like real Hikvision devices: a Basic challenge first, then Digest.
        format!(
            "HTTP/1.1 401 Unauthorized\r\n\
             WWW-Authenticate: Basic realm=\"{realm}\"\r\n\
             WWW-Authenticate: Digest realm=\"{realm}\", nonce=\"{nonce}\", qop=\"auth\"{stale}\r\n\
             Content-Length: 0\r\nConnection: close\r\n\r\n",
            realm = REALM,
            nonce = state.nonce(),
            stale = if stale { ", stale=\"TRUE\"" } else { "" },
        )
        .into_bytes()
    };

    stream.write_all(&response).unwrap();
}

/// Recomputes the digest the way a camera would and compares it.
fn verify(header: &AuthorizationHeader, method: &str, body: &[u8]) -> bool {
    let mut context = AuthContext::new_with_method(
        USERNAME,
        PASSWORD,
        header.uri.as_str(),
        Some(body),
        HttpMethod::from(method),
    );
    if let Some(cnonce) = &header.cnonce {
        context.set_custom_cnonce(cnonce.as_str());
    }

    let mut expected = header.clone();
    expected.digest(&context);

    header.realm == REALM && expected.response == header.response
}

pub const JPEG: &[u8] = b"\xff\xd8\xff\xe0fake-jpeg\xff\xd9";

fn response_status(code: i32, text: &str, sub: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ResponseStatus version="2.0" xmlns="http://www.hikvision.com/ver20/XMLSchema">
<requestURL>/ISAPI</requestURL>
<statusCode>{code}</statusCode>
<statusString>{text}</statusString>
<subStatusCode>{sub}</subStatusCode>
</ResponseStatus>"#
    )
}

fn http(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

/// Responses for authenticated requests.
fn route(method: &str, uri: &str) -> Vec<u8> {
    let path = uri.split('?').next().unwrap_or_default();

    match (method, path) {
        ("GET", "/ISAPI/System/deviceInfo") => {
            http("200 OK", "application/xml", DEVICE_INFO.as_bytes())
        }
        ("GET", "/ISAPI/Streaming/channels/1/picture") => http("200 OK", "image/jpeg", JPEG),
        // Firmware that answers 200 with XML instead of an image.
        ("GET", "/ISAPI/Streaming/channels/2/picture") => http(
            "200 OK",
            "application/xml",
            response_status(4, "Invalid Operation", "notSupport").as_bytes(),
        ),
        ("PUT", "/ISAPI/PTZCtrl/channels/1/presets/99/goto") => http(
            "403 Forbidden",
            "application/xml",
            response_status(4, "Invalid Operation", "invalidOperation").as_bytes(),
        ),
        // Failure reported only in the body of a 200.
        ("PUT", "/ISAPI/PTZCtrl/channels/1/presets/98/goto") => http(
            "200 OK",
            "application/xml",
            response_status(3, "Device Error", "deviceError").as_bytes(),
        ),
        ("PUT", _) => http(
            "200 OK",
            "application/xml",
            response_status(1, "OK", "ok").as_bytes(),
        ),
        _ => http("404 Not Found", "text/plain", b"not found"),
    }
}

/// A local port with nothing listening on it.
pub fn closed_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
