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

        match bytes[2] {
            I3LIDAR_NEW_DISTANCE => Ok(Packet::Distance),
            I3LIDAR_LIDAR_SPEED => Ok(Packet::LidarSpeed),
            command_byte => Err(PacketParseError::UnsupportedCommandByte(command_byte)),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum PacketParseError {
    #[error("The frame is too short: {0:}")]
    FrameTooShort(usize),
    #[error("Unsupported command byte: {0:}")]
    UnsupportedCommandByte(u8),
}
