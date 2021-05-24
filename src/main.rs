use anyhow::{Context, Result};
use delta_2a_lidar::Lidar;

fn main() -> Result<()> {
    println!("Enumerating lidars");
    let mut lidar_names = Lidar::enumerate()?;

    println!("Taking the first lidar");
    let lidar_name = lidar_names.next().context("Lidar was not found")?;

    println!("Connecting to: {}", lidar_name);
    let _lidar = Lidar::open(lidar_name)?;

    Ok(())
}
