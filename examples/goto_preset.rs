use std::env;

use hikvision_rs::{Camera, Client, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera {
        host: env::var("HOST")?,
        username: env::var("USERNAME")?,
        password: env::var("PASSWORD")?,
    };

    let channel: u32 = env::var("CHANNEL")?.parse()?;
    let preset_id: u32 = env::args()
        .nth(1)
        .ok_or("Missing preset_id argument")?
        .parse()?;

    let client = Client::new(ClientConfig::default())?;

    client.goto_preset(&camera, channel, preset_id)?;

    println!("Successfully moved to preset {}", preset_id);

    Ok(())
}
