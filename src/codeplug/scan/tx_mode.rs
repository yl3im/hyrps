use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ScanTxMode {
    Selected,
    LastActive,
    Designated,
}

impl TryFrom<u16> for ScanTxMode {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match (value >> 6) & 3 {
            0 => Ok(ScanTxMode::Selected),
            1 => Ok(ScanTxMode::LastActive),
            2 => Ok(ScanTxMode::Designated),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown Scan Tx Mode")),
        }
    }
}
