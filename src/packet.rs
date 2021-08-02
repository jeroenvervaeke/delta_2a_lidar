use crate::frame_parser::Frame;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

const I3LIDAR_NEW_DISTANCE: u8 = 0xAD;
const I3LIDAR_LIDAR_SPEED: u8 = 0xAE;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Packet {
    Distance(DistancePacket),
    LidarSpeed,
}

impl Packet {
    pub fn parse(frame: Frame) -> Result<Self, PacketParseError> {
        let bytes: Vec<_> = frame.into();
        let frame_len = bytes.len();

        if frame_len < 3 {
            return Err(PacketParseError::FrameTooShort(frame_len));
        }

        let command_byte = bytes[2];
        let data = &bytes[3..];

        match command_byte {
            I3LIDAR_NEW_DISTANCE => Self::parse_distance(data),
            I3LIDAR_LIDAR_SPEED => Self::parse_lidar_speed(data),
            command_byte => Err(PacketParseError::UnsupportedCommandByte(command_byte)),
        }
    }

    fn parse_distance(data: &[u8]) -> Result<Self, PacketParseError> {
        let actual_length = data.len();

        // A measuring package is at least 5 bytes long (header)
        if actual_length < 5 {
            return Err(PacketParseError::FrameTooShort(actual_length));
        }

        // Calculate the effective length
        let effective_data_length = ((data[0] as u16) << 8) | (data[1] as u16);
        let actual_data_length = (actual_length - 2) as u16;
        if effective_data_length != actual_data_length {
            return Err(PacketParseError::UnexpectedFrameLength {
                actual: actual_data_length,
                expected: effective_data_length,
            });
        }

        // Get the radar speed
        let radar_speed = 0.05f32 * data[2] as f32;

        // Calculate the offset angle
        let offset_angle = (((data[3] as u16) << 8) | (data[4] as u16)) as f32 * 0.01f32;

        // Calculate the start angle
        let start_angle = (((data[5] as u16) << 8) | (data[6] as u16)) as f32 * 0.01f32;

        // Convert all remaining bytes into data
        // Drop signal strength (only useful for sensor debugging purposes according to the datasheet)
        let measurements = data[7..]
            .chunks(3)
            .filter_map(|values| match values {
                [_signal_strength, value_high, value_low] => Some((((*value_high as u16) << 8) | (*value_low as u16)) as f32 * 0.25f32),
                _ => None,
            })
            .collect();

        // Return the distance packet
        Ok(Packet::Distance(DistancePacket {
            radar_speed,
            start_angle,
            offset_angle,
            measurements,
        }))
    }

    fn parse_lidar_speed(_data: &[u8]) -> Result<Self, PacketParseError> {
        todo!()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum PacketParseError {
    #[error("The frame is too short: {0:}")]
    FrameTooShort(usize),
    #[error("Unsupported command byte: {0:}")]
    UnsupportedCommandByte(u8),
    #[error("Unexpected frame length, expected {expected:}, got: {actual:}")]
    UnexpectedFrameLength { actual: u16, expected: u16 },
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct DistancePacket {
    radar_speed: f32,
    start_angle: f32,
    offset_angle: f32,
    measurements: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_parser::{FrameNextByteResult, FrameParser};
    use crate::mock_data::FIRST_EXAMPLE;

    fn first_example_package() -> DistancePacket {
        let frame = FIRST_EXAMPLE
            .iter()
            .fold(FrameNextByteResult::Unfinished(FrameParser::new()), |acc, current_byte| match acc {
                FrameNextByteResult::Finished(frame) => FrameNextByteResult::Finished(frame),
                FrameNextByteResult::Unfinished(frame_parser) => frame_parser.next_byte(*current_byte).expect("Should not fail"),
            })
            .finished()
            .unwrap();

        let packet = Packet::parse(frame).unwrap();

        match packet {
            Packet::Distance(distance_packet) => distance_packet,
            Packet::LidarSpeed => panic!("First example is distance, not lidar speed"),
        }
    }

    #[test]
    fn test_example_1_radar_speed() {
        let packet = first_example_package();

        assert_eq!(
            DistancePacket {
                radar_speed: 6.5f32,
                start_angle: 270.0f32,
                offset_angle: 1.35,
                measurements: vec![
                    0f32, 2126.5f32, 2270f32, 0f32, 0f32, 3288f32, 3261.75f32, 3258.75f32, 3256f32, 2146f32, 0f32, 2146f32, 2147.25f32, 2159.75f32, 3253f32,
                    3264.5f32, 3256f32, 5202f32, 5202f32, 5202f32, 5202f32, 5126.25f32, 5202f32, 5209f32, 5209f32, 5202f32, 5209f32, 5202f32, 5209f32,
                    5323.25f32, 5742.5f32, 3038f32, 3001f32, 0f32, 2999.5f32, 3001f32, 3028f32, 3001f32, 3033f32, 3038f32, 5762.75f32, 5887.25f32, 5876.75f32,
                    5898f32, 5898f32, 5887.25f32, 6028.5
                ],
            },
            packet
        );
    }
}
