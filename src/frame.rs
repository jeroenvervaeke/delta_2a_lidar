use thiserror::Error;

const FRAME_HEADER: u8 = 0xAA;

#[derive(Debug, PartialEq)]
pub enum Frame {
    // Step 1: Waiting for frame header
    // This value is fixed to 0xAA
    WaitingForHeader,

    // Step 2: Waiting for first byte of 2 bytes frame header length
    LengthPart1,

    // Step 3: Waiting for the 2nd byte of the 2 bytes frame header length
    LengthPart2 { length_part_1: u8 },

    // Step 4: Waiting for remaining bytes
    ReceiveFrameData { length: u16, data: Vec<u8> },
}

impl Frame {
    pub fn new() -> Self {
        Frame::WaitingForHeader
    }

    pub fn next_byte(&mut self, value: u8) -> Result<FrameReadResult, FrameParseError> {
        match self {
            Frame::WaitingForHeader => {
                if value == FRAME_HEADER {
                    *self = Frame::LengthPart1;

                    Ok(FrameReadResult::Incomplete)
                } else {
                    Err(FrameParseError::InvalidFrameHeader(value))
                }
            }
            Frame::LengthPart1 => {
                *self = Frame::LengthPart2 { length_part_1: value };

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
        assert_eq!(Frame::LengthPart1, frame);
    }

    #[test]
    fn frame_header_nok() {
        let mut frame = Frame::new();

        assert_eq!(Err(FrameParseError::InvalidFrameHeader(0x42)), frame.next_byte(0x42));
        assert_eq!(Frame::WaitingForHeader, frame);
    }

    #[test]
    fn frame_length_part_1_ok() {
        let mut frame = Frame::LengthPart1;

        assert_eq!(Ok(FrameReadResult::Incomplete), frame.next_byte(0x00));
        assert_eq!(Frame::LengthPart2 { length_part_1: 0x00 }, frame);
    }
}
