use crate::crc::CRC;
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

    pub fn next_byte(&mut self, value: u8) -> Result<FrameReadResult, FrameParseError> {
        match self {
            Frame::WaitingForHeader => {
                if value == FRAME_HEADER {
                    *self = Frame::LengthPart1 {
                        calculated_crc: CRC::new(FRAME_HEADER),
                    };

                    Ok(FrameReadResult::Incomplete)
                } else {
                    Err(FrameParseError::InvalidFrameHeader(value))
                }
            }
            Frame::LengthPart1 { calculated_crc } => {
                *self = Frame::LengthPart2 {
                    calculated_crc: calculated_crc.calculate_next(value),
                    length_part_1: value,
                };

                Ok(FrameReadResult::Incomplete)
            }
            Frame::LengthPart2 { calculated_crc, length_part_1 } => {
                let length = ((*length_part_1 as u16) << 8) + (value as u16);

                // First 3 bytes are included in the size but will not be found int the data Vec
                // First 3 bytes = Frame header (1B) + Frame length (2B)
                let capacity = length as usize - 3;

                *self = Frame::ReceiveFrameData {
                    calculated_crc: calculated_crc.calculate_next(value),
                    length,
                    data: Vec::with_capacity(capacity),
                };

                Ok(FrameReadResult::Incomplete)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FrameReadResult {
    Incomplete,
    Finished,
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
        let mut frame = Frame::new();

        assert_eq!(Ok(FrameReadResult::Incomplete), frame.next_byte(FRAME_HEADER));
        assert_eq!(
            Frame::LengthPart1 {
                calculated_crc: CRC::new(FRAME_HEADER)
            },
            frame
        );
    }

    #[test]
    fn frame_header_nok() {
        let mut frame = Frame::new();

        assert_eq!(Err(FrameParseError::InvalidFrameHeader(0x42)), frame.next_byte(0x42));
        assert_eq!(Frame::WaitingForHeader, frame);
    }

    #[test]
    fn frame_length_part_1_ok() {
        let mut frame = Frame::LengthPart1 {
            calculated_crc: CRC::new(FRAME_HEADER),
        };

        assert_eq!(Ok(FrameReadResult::Incomplete), frame.next_byte(0x00));
        assert_eq!(
            Frame::LengthPart2 {
                calculated_crc: CRC::new(FRAME_HEADER),
                length_part_1: 0x00
            },
            frame
        );
    }

    #[test]
    fn frame_length_part_2_ok() {
        let mut frame = Frame::LengthPart2 {
            calculated_crc: CRC::new(FRAME_HEADER),
            length_part_1: 0,
        };

        assert_eq!(Ok(FrameReadResult::Incomplete), frame.next_byte(0x09));
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
