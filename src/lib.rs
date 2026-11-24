//! Hikvision ISAPI SDK.
//!
//! Use [`Client`] (blocking) or [`AsyncClient`] (Tokio). Both authenticate with
//! HTTP digest auth and reuse the camera's challenge, so after the first call
//! each request is a single round-trip.

mod async_client;
mod auth;
pub mod client;
mod device;
pub mod error;
mod isapi;
pub mod models;

pub use async_client::AsyncClient;
pub use client::{Client, ClientConfig};
pub use device::{AsyncDevice, Device};
pub use error::{HikvisionError, Result};

pub use models::{
    Camera, DeviceInfo, Position, Preset, PresetList, ResponseStatus, Snapshot, SnapshotOptions,
};
