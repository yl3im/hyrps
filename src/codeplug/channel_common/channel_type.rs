use std::convert::TryFrom;
use std::io::Error;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ChannelType {
    Digial,
    Analog,
}

impl TryFrom<u8> for ChannelType {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Error> {
        match v {
            0x0 => Ok(ChannelType::Digial),
            0x1 => Ok(ChannelType::Analog),
            _ => Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid channel type",
            )),
        }
    }
}
