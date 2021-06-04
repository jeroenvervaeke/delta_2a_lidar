use crate::crc::CRC;
use thiserror::Error;

const FRAME_HEADER: u8 = 0xAA;

/// Enum used to capture Lidar frames.
///
/// A frame looks as follows:
///
/// |                   | Frame Header | Frame Length | Protocol version | Frame Type | Data                                       | CRC |
/// |------------------|-----------------------|----------------------|-------------------------|--------------------|-----------------------------------------------|---------|
/// | Fixed         | Yes                | No                 | Yes                  | Yes              |  No                                         | No    |
/// | Value/Size | 0xAA              | 2B                 | 0x00                 | 0x61            |  Frame length - 2 (at least 1B)   | 2B    |
#[derive(Debug, PartialEq)]
pub enum FrameParser {
    /// Step 1: Waiting for frame header
    ///
    /// This value is fixed to 0xAA
    WaitingForHeader,

    /// Step 2: Waiting for first byte of 2 bytes frame header length
    LengthPart1 { calculated_crc: CRC },

    /// Step 3: Waiting for the 2nd byte of the 2 bytes frame header length
    LengthPart2 { calculated_crc: CRC, length_part_1: u8 },

    /// Step 4: Waiting for remaining bytes
    ReceiveFrameData { calculated_crc: CRC, length: u16, data: Vec<u8> },

    /// Step 5: Waiting for first byte of CRC
    CRCPart1 { calculated_crc: CRC, data: Vec<u8> },

    /// Step 6: Waiting for second byte of CRC
    CRCPart2 {
        calculated_crc: CRC,
        data: Vec<u8>,
        received_crc_part_1: u8,
    },
}

impl FrameParser {
    /// Create a new frame parser
    pub fn new() -> Self {
        FrameParser::WaitingForHeader
    }

    /// Feed the next byte into a frame and calculate the resulting frame
    pub fn next_byte(self, value: u8) -> Result<FrameNextByteResult, FrameParseError> {
        match self {
            FrameParser::WaitingForHeader => {
                if value == FRAME_HEADER {
                    Ok(FrameNextByteResult::Unfinished(FrameParser::LengthPart1 {
                        calculated_crc: CRC::new(FRAME_HEADER),
                    }))
                } else {
                    Err(FrameParseError::InvalidFrameHeader(value))
                }
            }
            FrameParser::LengthPart1 { calculated_crc } => Ok(FrameNextByteResult::Unfinished(FrameParser::LengthPart2 {
                calculated_crc: calculated_crc.calculate_next(value),
                length_part_1: value,
            })),
            FrameParser::LengthPart2 { calculated_crc, length_part_1 } => {
                let length = ((length_part_1 as u16) << 8) + (value as u16);

                // If the length is < 6 consider the frame to be invalid
                // See docs of frame why.
                if length < 6 {
                    return Err(FrameParseError::InvalidFrameLength(length));
                }

                // First 3 bytes are included in the size but will not be found int the data Vec
                // First 3 bytes = Frame header (1B) + Frame length (2B)
                let capacity = length as usize - 3;

                Ok(FrameNextByteResult::Unfinished(FrameParser::ReceiveFrameData {
                    calculated_crc: calculated_crc.calculate_next(value),
                    length,
                    data: Vec::with_capacity(capacity),
                }))
            }
            FrameParser::ReceiveFrameData {
                calculated_crc,
                length,
                mut data,
            } => {
                // Append the new data
                data.push(value);

                // Calculate the new crc
                let calculated_crc = calculated_crc.calculate_next(value);

                // Calculate the actual length
                // Data length + Frame Header (1B) + Frame Length (2B)
                let actual_length = data.len() as u16 + 3;

                // When we reached the frame length move on to CRC parsing
                let next_frame = if actual_length == length {
                    FrameParser::CRCPart1 { calculated_crc, data }
                } else {
                    FrameParser::ReceiveFrameData { calculated_crc, length, data }
                };

                // Return the new frame
                Ok(FrameNextByteResult::Unfinished(next_frame))
            }
            FrameParser::CRCPart1 { calculated_crc, data } => Ok(FrameNextByteResult::Unfinished(FrameParser::CRCPart2 {
                calculated_crc,
                data,
                received_crc_part_1: value,
            })),
            FrameParser::CRCPart2 {
                calculated_crc,
                data,
                received_crc_part_1,
            } => {
                let received = ((received_crc_part_1 as u16) << 8) + value as u16;
                let calculated = calculated_crc.as_u16();

                if received != calculated {
                    Err(FrameParseError::InvalidCRC { calculated, received })
                } else {
                    Ok(FrameNextByteResult::Finished(Frame(data)))
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FrameNextByteResult {
    Unfinished(FrameParser),
    Finished(Frame),
}

impl FrameNextByteResult {
    pub fn finished(self) -> Option<Frame> {
        match self {
            FrameNextByteResult::Unfinished(_) => None,
            FrameNextByteResult::Finished(f) => Some(f),
        }
    }

    pub fn unfinished(self) -> Option<FrameParser> {
        match self {
            FrameNextByteResult::Unfinished(u) => Some(u),
            FrameNextByteResult::Finished(_) => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Frame(Vec<u8>);

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum FrameParseError {
    #[error("Invalid frame header, expected 0xAA, got {0:#X}")]
    InvalidFrameHeader(u8),
    #[error("Invalid frame length, expected a length > 5")]
    InvalidFrameLength(u16),
    #[error("Invalid CRC, calculated {calculated:#X}, received {received:#X}")]
    InvalidCRC { calculated: u16, received: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_data::*;

    #[test]
    fn frame_header_ok() {
        let frame = FrameParser::new();

        assert_eq!(
            Ok(FrameNextByteResult::Unfinished(FrameParser::LengthPart1 {
                calculated_crc: CRC::new(FRAME_HEADER)
            })),
            frame.next_byte(FRAME_HEADER)
        );
    }

    #[test]
    fn frame_header_nok() {
        let frame = FrameParser::new();

        assert_eq!(Err(FrameParseError::InvalidFrameHeader(0x42)), frame.next_byte(0x42));
    }

    #[test]
    fn frame_length_part_1_ok() {
        let frame = FrameParser::LengthPart1 {
            calculated_crc: CRC::new(FRAME_HEADER),
        };

        assert_eq!(
            Ok(FrameNextByteResult::Unfinished(FrameParser::LengthPart2 {
                calculated_crc: CRC::new(FRAME_HEADER),
                length_part_1: 0x00
            })),
            frame.next_byte(0x00)
        );
    }

    #[test]
    fn frame_length_part_2_ok() {
        let frame = FrameParser::LengthPart2 {
            calculated_crc: CRC::new(FRAME_HEADER),
            length_part_1: 0,
        };

        let frame = frame.next_byte(0x09).unwrap().unfinished().unwrap();

        assert_eq!(
            FrameParser::ReceiveFrameData {
                calculated_crc: CRC::new(0xB3), // 0xAA + 0x00 + 0x09
                length: 0x0009,
                data: vec![]
            },
            frame
        );

        match frame {
            FrameParser::ReceiveFrameData { data, .. } if data.capacity() == 6 => { /* valid */ }
            FrameParser::ReceiveFrameData { data, .. } => panic!("Invalid data capacity: {}", data.capacity()),
            _ => panic!("Invalid state!"),
        }
    }

    #[test]
    fn frame_length_part_2_nok() {
        let frame = FrameParser::LengthPart2 {
            calculated_crc: CRC::new(FRAME_HEADER),
            length_part_1: 0,
        };

        assert_eq!(Err(FrameParseError::InvalidFrameLength(0x03)), frame.next_byte(0x03));
    }

    #[test]
    fn receive_frame_data_ok() {
        let frame = FrameParser::ReceiveFrameData {
            calculated_crc: CRC::from_u16(0xAD), // 0xAA + 0x00 + 0x03
            length: 6,
            data: vec![],
        };

        // Protocol version
        let after_step_1 = frame.next_byte(0x00).unwrap().unfinished().unwrap();
        assert_eq!(
            FrameParser::ReceiveFrameData {
                calculated_crc: CRC::from_u16(0xAD), // 0xAA + 0x00 + 0x03 + 0x00
                length: 6,
                data: vec![0x00],
            },
            after_step_1
        );

        // Frame type
        let after_step_2 = after_step_1.next_byte(0x61).unwrap().unfinished().unwrap();
        assert_eq!(
            FrameParser::ReceiveFrameData {
                calculated_crc: CRC::from_u16(0x10E), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61
                length: 6,
                data: vec![0x00, 0x61],
            },
            after_step_2
        );

        // Data: invalid special unit test type
        let after_step_3 = after_step_2.next_byte(0xFA).unwrap().unfinished().unwrap();
        assert_eq!(
            FrameParser::CRCPart1 {
                calculated_crc: CRC::from_u16(0x208), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61 + FA
                data: vec![0x00, 0x61, 0xFA],
            },
            after_step_3
        );
    }

    #[test]
    fn test_crc_part_1() {
        let frame = FrameParser::CRCPart1 {
            calculated_crc: CRC::from_u16(0x208), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61 + FA
            data: vec![0x00, 0x61, 0xFA],
        };

        assert_eq!(
            Ok(FrameNextByteResult::Unfinished(FrameParser::CRCPart2 {
                calculated_crc: CRC::from_u16(0x208), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61 + FA
                data: vec![0x00, 0x61, 0xFA],
                received_crc_part_1: 0x02
            })),
            frame.next_byte(0x02)
        );
    }

    #[test]
    fn test_crc_part_2_ok() {
        let frame = FrameParser::CRCPart2 {
            calculated_crc: CRC::from_u16(0x208), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61 + FA
            data: vec![0x00, 0x61, 0xFA],
            received_crc_part_1: 0x02,
        };

        assert_eq!(Ok(FrameNextByteResult::Finished(Frame(vec![0x00, 0x61, 0xFA]))), frame.next_byte(0x08));
    }

    #[test]
    fn test_crc_part_2_nok() {
        let frame = FrameParser::CRCPart2 {
            calculated_crc: CRC::from_u16(0x208), // 0xAA + 0x00 + 0x03 + 0x00 + 0x61 + FA
            data: vec![0x00, 0x61, 0xFA],
            received_crc_part_1: 0x02,
        };

        assert_eq!(
            Err(FrameParseError::InvalidCRC {
                calculated: 0x208,
                received: 0x207
            }),
            frame.next_byte(0x07)
        );
    }

    fn test_example(example_bytes: &[u8]) {
        let result = example_bytes
            .iter()
            .fold(FrameNextByteResult::Unfinished(FrameParser::new()), |acc, current_byte| match acc {
                FrameNextByteResult::Finished(frame) => FrameNextByteResult::Finished(frame),
                FrameNextByteResult::Unfinished(frame_parser) => frame_parser.next_byte(*current_byte).expect("Should not fail"),
            });

        assert!(matches!(result, FrameNextByteResult::Finished(_)));
    }

    #[test]
    fn example_1() {
        test_example(&FIRST_EXAMPLE);
    }

    #[test]
    fn example_2() {
        test_example(&SECOND_EXAMPLE);
    }
}
