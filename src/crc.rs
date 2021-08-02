/// The CRC type is used to calculate the CRC of a lidar package.
/// The underlying type for the CRC is an u16.
///
/// The CRC calculation is very straightforward. It's a sum of all bytes.
/// When the underlying u16 overflows it is ignored (hence the use of overflowing_add)
#[derive(Debug, PartialEq)]
pub struct CRC(u16);

impl CRC {
    /// Create a new CRC from an u8
    pub fn new(value: u8) -> CRC {
        CRC(value as u16)
    }

    /// Create a new CRC from an u16
    pub fn from_u16(value: u16) -> CRC {
        CRC(value)
    }

    /// Calculate the CRC from an iterator of bytes
    pub fn from_bytes<'a, I: IntoIterator<Item = &'a u8>>(bytes: I) -> CRC {
        let iter = bytes.into_iter();
        iter.fold(CRC::new(0), |acc, current| acc.calculate_next(*current))
    }

    /// Calculate the new CRC based on the current CRC and the next byte (u8)
    pub fn calculate_next(&self, value: u8) -> CRC {
        let next_value = self.0.overflowing_add(value as u16).0;
        CRC(next_value)
    }

    /// Get the CRC value as an u16
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_data::*;

    fn test_example(example: &[u8], expected_crc: u16) {
        // Ignore the last 2 bytes because they're part of the CRC
        let crc = CRC::from_bytes(example.iter().take(example.len() - 2));
        assert_eq!(expected_crc, crc.as_u16())
    }

    #[test]
    fn example_1() {
        test_example(&FIRST_EXAMPLE, 0x35BC);
    }

    #[test]
    fn example_2() {
        test_example(&SECOND_EXAMPLE, 0x022C);
    }
}
