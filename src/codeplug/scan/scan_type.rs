use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ScanType {
    Normal,
    Vote,
    DigitalChannel,
}

impl TryFrom<u16> for ScanType {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value & 3 {
            0 => Ok(ScanType::Normal),
            1 => Ok(ScanType::Vote),
            2 => Ok(ScanType::DigitalChannel),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown Scan Type")),
        }
    }
}
