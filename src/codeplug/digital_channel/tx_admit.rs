use core::convert::TryFrom;
#[cfg(test)]
use proptest_derive::Arbitrary;
use std::io::Error;
use std::io::ErrorKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum TxAdmit {
    Always = 0,
    Channel = 1,
    ColourCode = 2,
}

impl TryFrom<u8> for TxAdmit {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v & 0x3 {
            0 => Ok(TxAdmit::Always),
            1 => Ok(TxAdmit::Channel),
            2 => Ok(TxAdmit::ColourCode),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown TxAdmit type 0x{:X}", v),
            )),
        }
    }
}
