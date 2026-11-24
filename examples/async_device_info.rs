use std::env;

use hikvision_rs::{AsyncClient, Camera, ClientConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::new(
        env::var("HOST")?,
        env::var("USERNAME")?,
        env::var("PASSWORD")?,
    );

    let client = AsyncClient::new(ClientConfig::default())?;
    let device = client.device(camera);

    let info = device.get_device_info().await?;

    println!("Model   : {:?}", info.model);
    println!("Firmware: {:?}", info.firmware_version);

    Ok(())
}
