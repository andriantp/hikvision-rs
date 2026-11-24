use std::env;
use std::fs;

use hikvision_rs::{Camera, Client, ClientConfig, SnapshotOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera {
        host: env::var("HOST")?,
        username: env::var("USERNAME")?,
        password: env::var("PASSWORD")?,
    };

    let channel: u32 = env::var("CHANNEL")?.parse()?;

    let options = SnapshotOptions::default();

    let client = Client::new(ClientConfig::default())?;

    let snapshot = client.get_snapshot(&camera, channel, options)?;

    let filename = format!("examples/snapshot_channel_{}.jpg", channel);
    fs::write(&filename, &snapshot.data)?;

    println!("Snapshot captured");
    println!("Content-Type: {}", snapshot.content_type);
    println!("Image size: {} bytes", snapshot.data.len());
    println!("Saved to: {}", filename);

    Ok(())
}
