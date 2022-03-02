use super::super::cp_data::RawCPData;
use byteorder::{ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryInto;
use std::{convert::TryFrom, io::Read};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CTCSSType {
    None = 0,
    Ctcss = 1,
    Cdcss = 2,
    CdcssInvert = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Ctcss {
    pub kind: CTCSSType,
    #[cfg_attr(test, proptest(strategy = "0..4096u16"))]
    pub freq: u16,
}

impl RawCPData for Ctcss {
    fn load(reader: &mut impl Read) -> anyhow::Result<Ctcss> {
        let mut freq = (reader.read_u8()?) as u16;
        let b = reader.read_u8()?;

        freq += ((b & 0xf) as u16) << 8;

        let kind = CTCSSType::try_from(b >> 6)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(Ctcss { kind, freq })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        writer.write_u8((self.freq & 0xff).try_into().unwrap())?;

        let mut b: u8 = (self.freq >> 8).try_into().unwrap();

        b |= (self.kind as u8) << 6;

        writer.write_u8(b)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn emerg_serial_deserialise(ctcss in any::<super::Ctcss>()) {
            check_serde(&ctcss)?;
        }
    }
}
