use delta_2a_lidar::Lidar;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Enumerating lidars");

    for lidar in Lidar::enumerate()? {
        println!("Lidar name: {}", lidar);
    }

    println!("Finished enumerating");

    Ok(())
}