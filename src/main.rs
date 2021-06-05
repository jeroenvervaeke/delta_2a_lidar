use anyhow::{Context, Result};
use delta_2a_lidar::Lidar;
use log::info;
use pretty_env_logger::env_logger::{Builder, Env};

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Enumerating lidars");
    let mut lidar_names = Lidar::enumerate()?;

    info!("Taking the first lidar");
    let lidar_name = lidar_names.next().context("Lidar was not found")?;

    info!("Connecting to: {}", lidar_name);
    let mut lidar = Lidar::open(lidar_name)?;

    while let Some(package) = lidar.next().await {
        info!("Received package: {:?}", package);
    }

    info!("Finished receiving messages, quitting");

    Ok(())
}
