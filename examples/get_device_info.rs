use std::env;

use hikvision_rs::{Camera, Client, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera {
        host: env::var("HOST")?,
        username: env::var("USERNAME")?,
        password: env::var("PASSWORD")?,
    };

    let client = Client::new(ClientConfig::default())?;

    let device_info = client.get_device_info(&camera)?;

    println!("Device Information");
    println!("------------------");
    println!("Name       : {:?}", device_info.device_name);
    println!("ID         : {:?}", device_info.device_id);
    println!("Description: {:?}", device_info.device_description);
    println!("Location   : {:?}", device_info.device_location);
    println!("Model      : {:?}", device_info.model);
    println!("Serial     : {:?}", device_info.serial_number);
    println!("Firmware   : {:?}", device_info.firmware_version);

    Ok(())
}
