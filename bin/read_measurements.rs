use anyhow::Result;
use delta_2a_lidar::{measurements_file, packet_stream::PacketStream};
use log::info;
use pretty_env_logger::env_logger::{Builder, Env};

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Read measurement file");
    let mut measurements = measurements_file::read("./measurements.ldr").await?;

    while let Some(package) = measurements.next().await {
        info!("Received package: {:?}", package);
    }

    info!("Finished receiving messages, quitting");

    Ok(())
}
