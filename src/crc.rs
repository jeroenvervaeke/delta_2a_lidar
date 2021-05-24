#[derive(Debug, PartialEq)]
pub struct CRC(u16);

impl CRC {
    pub fn new(value: u8) -> CRC {
        CRC(value as u16)
    }

    pub fn from_u16(value: u16) -> CRC {
        CRC(value)
    }

    pub fn from_bytes<'a, I: IntoIterator<Item = &'a u8>>(bytes: I) -> CRC {
        let iter = bytes.into_iter();
        iter.fold(CRC::new(0), |acc, current| acc.calculate_next(*current))
    }

    pub fn calculate_next(&self, value: u8) -> CRC {
        let next_value = self.0.overflowing_add(value as u16).0;
        CRC(next_value)
    }

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
