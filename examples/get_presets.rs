use std::env;

use hikvision_rs::{Camera, Client, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera {
        host: env::var("HOST")?,
        username: env::var("USERNAME")?,
        password: env::var("PASSWORD")?,
    };

    let channel: u32 = env::var("CHANNEL")?.parse()?;

    let client = Client::new(ClientConfig::default())?;

    let presets = client.get_presets(&camera, channel)?;

    println!("PTZ Presets");
    println!("-----------");

    for preset in presets {
        println!("ID        : {}", preset.id);
        println!("Name      : {}", preset.name);
        match &preset.position {
            Some(position) => {
                println!("Elevation : {}", position.elevation);
                println!("Azimuth   : {}", position.azimuth);
                println!("Zoom      : {}", position.zoom);
            }
            None => println!("Position  : (not reported)"),
        }
        println!();
    }

    Ok(())
}
