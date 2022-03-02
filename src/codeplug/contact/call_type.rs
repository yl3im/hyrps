use std::convert::TryFrom;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CallType {
    Private = 0,
    Group = 1,
    Ignore = 0x11,
}

impl TryFrom<u8> for CallType {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CallType::Private),
            1 => Ok(CallType::Group),
            0x11 => Ok(CallType::Ignore),
            _ => Err(Self::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown call type {}", value),
            )),
        }
    }
}
