use crate::frame_parser::Frame;
use thiserror::Error;

const I3LIDAR_NEW_DISTANCE: u8 = 0xAD;
const I3LIDAR_LIDAR_SPEED: u8 = 0xAE;

#[derive(Debug, PartialEq)]
pub enum Packet {
    Distance,
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

    fn parse_distance(_data: &[u8]) -> Result<Self, PacketParseError> {
        todo!()
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
}
