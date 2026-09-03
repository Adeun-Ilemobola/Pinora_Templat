use crate::Unsigned32;

impl From<u32> for Unsigned32 {
    fn from(value: u32) -> Self {
        Self {
            high_word: ((value >> 16) & u16::MAX as u32) as i32,
            low_word: (value & u16::MAX as u32) as i32,
        }
    }
}

impl TryFrom<Unsigned32> for u32 {
    type Error = &'static str;

    fn try_from(value: Unsigned32) -> Result<Self, Self::Error> {
        let high_word =
            u16::try_from(value.high_word).map_err(|_| "Unsigned32.high_word must fit in u16")?;
        let low_word =
            u16::try_from(value.low_word).map_err(|_| "Unsigned32.low_word must fit in u16")?;

        Ok((u32::from(high_word) << 16) | u32::from(low_word))
    }
}
