# Delta-2A lidar driver
[![Build Status][build-img]][build-url]
[![Crates.io][crates-io-img]][crates-io-url]
[![Documentation][docs-img]][docs-url]
## About
This crate contains a rust driver implementation for the [3irobotix delta-2A Lidar Sensor](https://www.banggood.com/custlink/KG3dehcdKd).

## Features
- Read distance frames
- Read lidar speed (WIP)
- Read/write measurements to file + abstractions to mock sensor (behind `file` feature)

## Dependencies
This library uses the `serialport` crate which requires `libudev-dev` to be installed on your system.
__On Ubuntu:__
```sh
sudo apt-get update && sudo apt-get install -y libudev-dev
```

## Examples
### List al lidar sensors
This simple example prints all found lidar sensors.
```rust
use delta_2a_lidar::Lidar;

for sensor in Lidar::enumerate().unwrap() {
  println!("Found lidar sensor: {}", sensor);
}
```

### Read incoming packages
This simple example prints all incoming packages from the first lidar sensor we find
```rust
use delta_2a_lidar::Lidar;

// Get all lidars
let mut lidar_names = Lidar::enumerate().unwrap();

// Take the first lidar
let lidar_name = lidar_names.next().unwrap();

// Open the lidar
let mut lidar = Lidar::open(lidar_name).unwrap();

// Read packages as long as the lidar produces packages
while let Some(package) = lidar.next().await {
  println!("Received package: {:?}", package);
}
```


[build-img]: https://github.com/jeroenvervaeke/delta_2a_lidar/actions/workflows/build_and_test.yml/badge.svg?branch=master
[build-url]: https://github.com/jeroenvervaeke/delta_2a_lidar/actions/workflows/build_and_test.yml
[crates-io-img]: https://img.shields.io/crates/v/delta_2a_lidar.svg
[crates-io-url]: https://crates.io/crates/delta_2a_lidar
[docs-img]: https://docs.rs/delta_2a_lidar/badge.svg
[docs-url]: https://docs.rs/delta_2a_lidar
