use crate::frame_parser::Frame;
use thiserror::Error;

const I3LIDAR_NEW_DISTANCE: u8 = 0xAD;
const I3LIDAR_LIDAR_SPEED: u8 = 0xAE;

#[derive(Debug, PartialEq)]
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
        //let _effective_data_length = ((data[0] as u16) << 8) | (data[1] as u16);
        let radar_speed = 0.05f32 * data[2] as f32;

        if data[3] != 00 || data[4] != 0x87 {
            return Err(PacketParseError::ZeroPointOffsetIsMissing);
        }

        let start_angle = (((data[5] as u16) << 8) | (data[6] as u16)) as f32 * 0.01f32;

        println!("Length: {}", data.len());

        Ok(Packet::Distance(DistancePacket { radar_speed, start_angle }))
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
    #[error("ZeroPointOffsetIsMissing")]
    ZeroPointOffsetIsMissing,
}

#[derive(Debug, PartialEq)]
pub struct DistancePacket {
    radar_speed: f32,
    start_angle: f32,
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
                start_angle: 270.0f32
            },
            packet
        );
    }
}
