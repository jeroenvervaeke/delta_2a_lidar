use anyhow::{Context, Result};
use delta_2a_lidar::Lidar;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Enumerating lidars");
    let mut lidar_names = Lidar::enumerate()?;

    println!("Taking the first lidar");
    let lidar_name = lidar_names.next().context("Lidar was not found")?;

    println!("Connecting to: {}", lidar_name);
    let mut lidar = Lidar::open(lidar_name)?;

    while let Some(package) = lidar.next().await {
        println!("Received package: {:?}", package);
    }

    println!("Finished receiving messages, quitting");

    Ok(())
}
