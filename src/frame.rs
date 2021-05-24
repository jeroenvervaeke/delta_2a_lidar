use crate::crc::CRC;
use std::borrow::BorrowMut;
use thiserror::Error;

const FRAME_HEADER: u8 = 0xAA;

#[derive(Debug, PartialEq)]
pub enum Frame {
    // Step 1: Waiting for frame header
    // This value is fixed to 0xAA
    WaitingForHeader,

    // Step 2: Waiting for first byte of 2 bytes frame header length
    LengthPart1 {
        calculated_crc: CRC,
    },

    // Step 3: Waiting for the 2nd byte of the 2 bytes frame header length
    LengthPart2 {
        calculated_crc: CRC,
        length_part_1: u8,
    },

    // Step 4: Waiting for remaining bytes
    ReceiveFrameData {
        calculated_crc: CRC,
        length: u16,
        data: Vec<u8>,
    },

    // Step 5: Waiting for first byte of CRC
    CRCPart1 {
        calculated_crc: CRC,
        length: u16,
        data: Vec<u8>,
    },

    // Step 6: Waiting for second byte of CRC
    CRCPart2 {
        calculated_crc: CRC,
        length: u16,
        data: Vec<u8>,
        received_crc_part_1: u8,
    },
}

impl Frame {
    pub fn new() -> Self {
        Frame::WaitingForHeader
    }

    pub fn next_byte(self, value: u8) -> Result<Frame, FrameParseError> {
        match self {
            Frame::WaitingForHeader => {
                if value == FRAME_HEADER {
                    Ok(Frame::LengthPart1 {
                        calculated_crc: CRC::new(FRAME_HEADER),
                    })
                } else {
                    Err(FrameParseError::InvalidFrameHeader(value))
                }
            }
            Frame::LengthPart1 { calculated_crc } => Ok(Frame::LengthPart2 {
                calculated_crc: calculated_crc.calculate_next(value),
                length_part_1: value,
            }),
            Frame::LengthPart2 { calculated_crc, length_part_1 } => {
                let length = ((length_part_1 as u16) << 8) + (value as u16);

                // First 3 bytes are included in the size but will not be found int the data Vec
                // First 3 bytes = Frame header (1B) + Frame length (2B)
                let capacity = length as usize - 3;

                Ok(Frame::ReceiveFrameData {
                    calculated_crc: calculated_crc.calculate_next(value),
                    length,
                    data: Vec::with_capacity(capacity),
                })
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum FrameParseError {
    #[error("Invalid frame header, expected 0xAA, got {0:#X}")]
    InvalidFrameHeader(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_header_ok() {
        let frame = Frame::new();

        assert_eq!(
            Ok(Frame::LengthPart1 {
                calculated_crc: CRC::new(FRAME_HEADER)
            }),
            frame.next_byte(FRAME_HEADER)
        );
    }

    #[test]
    fn frame_header_nok() {
        let frame = Frame::new();

        assert_eq!(Err(FrameParseError::InvalidFrameHeader(0x42)), frame.next_byte(0x42));
    }

    #[test]
    fn frame_length_part_1_ok() {
        let frame = Frame::LengthPart1 {
            calculated_crc: CRC::new(FRAME_HEADER),
        };

        assert_eq!(
            Ok(Frame::LengthPart2 {
                calculated_crc: CRC::new(FRAME_HEADER),
                length_part_1: 0x00
            }),
            frame.next_byte(0x00)
        );
    }

    #[test]
    fn frame_length_part_2_ok() {
        let frame = Frame::LengthPart2 {
            calculated_crc: CRC::new(FRAME_HEADER),
            length_part_1: 0,
        };

        let frame = frame.next_byte(0x09).unwrap();

        assert_eq!(
            Frame::ReceiveFrameData {
                calculated_crc: CRC::new(0xB3), // 0xAA + 0x00 + 0x09
                length: 0x0009,
                data: vec![]
            },
            frame
        );

        match frame {
            Frame::ReceiveFrameData { data, .. } if data.capacity() == 6 => { /* valid */ }
            Frame::ReceiveFrameData { data, .. } => panic!("Invalid data capacity: {}", data.capacity()),
            _ => panic!("Invalid state!"),
        }
    }
}
