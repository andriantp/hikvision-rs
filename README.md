# hikvision-rs

A Rust SDK for interacting with Hikvision cameras through ISAPI (Internet Server Application Programming Interface).

## Features

- HTTP Digest Authentication (the challenge is reused, so after the first call each request is a single round-trip)
- Blocking (`Client`) and async (`AsyncClient`, Tokio) clients
- Device information retrieval
- PTZ preset retrieval and navigation
- JPEG snapshot capture
- Structured ISAPI errors (`ResponseStatus`)
- Pure-Rust TLS (rustls), optional acceptance of self-signed certificates

## Requirements

- Rust 1.85 or newer (edition 2024)
- For `AsyncClient`: a Tokio runtime in your application

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hikvision-rs = { git = "https://github.com/andriantp/hikvision-rs" }
```

## Usage

```rust
use hikvision_rs::{Camera, Client, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(ClientConfig::default())?;

    // Bind the client to one camera; `http://` is assumed if no scheme is given.
    let cam = client.device(Camera::new("http://your-camera-address:port", "your-username", "your-password"));

    let device_info = cam.get_device_info()?;
    println!("Model: {:?}", device_info.model);

    for preset in cam.get_presets(1)? {
        println!("{}: {}", preset.id, preset.name);
    }

    cam.goto_preset(1, 2)?;

    Ok(())
}
```

Channel numbers: PTZ endpoints use the channel number (usually `1`). For
snapshots, many devices also accept `101` (main stream) / `102` (sub stream),
and NVRs number channels per camera (`101`, `201`, ...).

Every method is also available on `Client` directly with a `&Camera` argument,
e.g. `client.get_presets(&camera, 1)`. One `Client` can serve many cameras;
clones share the connection pool and the digest cache.

### Snapshot

```rust
use hikvision_rs::SnapshotOptions;
use std::fs;

let snapshot = cam.get_snapshot(1, SnapshotOptions::default())?;

fs::write("snapshot.jpg", &snapshot.data)?;
```

### Async

Requires Tokio in your application, e.g.
`tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`.

```rust
use hikvision_rs::{AsyncClient, Camera, ClientConfig};

let client = AsyncClient::new(ClientConfig::default())?;
let cam = client.device(Camera::new("http://your-camera-address:port", "your-username", "your-password"));

let info = cam.get_device_info().await?;
```

Do not call the blocking `Client` from inside an async runtime (reqwest panics
there); use `AsyncClient`.

### HTTPS with a self-signed certificate

```rust
use hikvision_rs::ClientConfig;

let config = ClientConfig {
    accept_invalid_certs: true, // disables certificate verification
    ..ClientConfig::default()
};
```

### Errors

Rejections from the camera are returned as `HikvisionError::Api` with the parsed
`ResponseStatus`, also when the camera reports a failure inside a `200 OK`:

```rust
match cam.goto_preset(1, 99) {
    Err(err) if err.sub_status_code() == Some("notSupport") => println!("not supported"),
    Err(err) => println!("{err}"), // e.g. "ISAPI error (403): Invalid Operation (4), invalidOperation"
    Ok(()) => {}
}
```

## Examples

The examples read the camera from environment variables. Copy `.env.example`
to `.env`, fill it in, and load it into your shell:

```bash
cp .env.example .env        # then edit .env
source .env

cargo run --example get_device_info
cargo run --example get_presets
cargo run --example goto_preset -- 2      # preset id
cargo run --example get_snapshot          # writes examples/snapshot_channel_<CHANNEL>.jpg
cargo run --example async_device_info
```

| Variable   | Example               | Used by                        |
|------------|-----------------------|--------------------------------|
| `HOST`     | `http://192.168.1.64` | all                            |
| `USERNAME` | `admin`               | all                            |
| `PASSWORD` | `change-me`           | all                            |
| `CHANNEL`  | `1`                   | presets, goto, snapshot        |

Note: many Linux desktops already set `USERNAME` to your login name. `source .env`
overrides it for that shell, but in a new terminal where `.env` was not loaded
the examples will silently use your login name and fail with `401`.

`.env` and the snapshot images are git-ignored.

## Development

```bash
cargo test                  # unit + integration tests (local mock camera, no device needed)
cargo clippy --all-targets
cargo fmt
```

## License

MIT
