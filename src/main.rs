use delta_2a_lidar::Lidar;
use anyhow::{Result, Context};

fn main() -> Result<()> {
    println!("Enumerating lidars");
    let mut lidar_names = Lidar::enumerate()?;

    println!("Taking the first lidar");
    let lidar_name = lidar_names.next().context("Lidar was not found")?;

    println!("Connecting to: {}", lidar_name);
    let lidar = Lidar::open(lidar_name)?;

    Ok(())
}