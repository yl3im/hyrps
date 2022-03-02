use anyhow::{anyhow, bail};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::{convert::TryFrom, io::Read};

use crate::codeplug::{cp_data::RawCPData, Codeplug};

use super::{pointer::ChannelPointer, raw_pointer::RawPointer};

#[cfg(test)]
use proptest::strategy::Strategy;

#[cfg(test)]
fn idx_strategy() -> impl Strategy<Value = u16> {
    0..(u16::MAX - 1)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum DigiChannelPointer {
    Selected,
    #[cfg_attr(
        test,
        proptest(strategy = "idx_strategy().prop_map(DigiChannelPointer::Digital)")
    )]
    Digital(u16),
}

impl DigiChannelPointer {
    pub fn get_chan_name(&self, codeplug: &Codeplug) -> String {
        ChannelPointer::from(self).get_chan_name(codeplug)
    }
}

impl RawCPData for DigiChannelPointer {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let idx = reader.read_u16::<LittleEndian>()?;

        if idx == 0xffff {
            return Ok(Self::Selected);
        }

        Ok(Self::Digital(idx - 1))
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let idx_ser = match self {
            Self::Selected => 0xffff,
            Self::Digital(i) => i + 1,
        };

        writer.write_u16::<LittleEndian>(idx_ser)?;

        Ok(())
    }
}

impl TryFrom<&ChannelPointer> for DigiChannelPointer {
    type Error = anyhow::Error;

    fn try_from(value: &ChannelPointer) -> Result<Self, Self::Error> {
        Ok(match value {
            ChannelPointer::Selected => DigiChannelPointer::Selected,
            ChannelPointer::Digital(i) => DigiChannelPointer::Digital(*i),
            ChannelPointer::Analog(_) => {
                bail!("Could not convert {:?} to digi channel pointer", value)
            }
        })
    }
}

impl RawPointer for DigiChannelPointer {
    fn sz() -> usize {
        0x2
    }

    fn verify(&self, cp: &Codeplug) -> anyhow::Result<()> {
        match self {
            Self::Selected => Ok(()),
            Self::Digital(i) => cp
                .digi_chans
                .data
                .get(*i as usize)
                .ok_or_else(|| anyhow!("Invalid {} ({})", std::any::type_name::<Self>(), i))
                .map(|_| ()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::ChannelPointer;
    use super::*;
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn digi_channel_pointer_serde(dcp in any::<DigiChannelPointer>()) {
            check_serde(&dcp)?;
        }
    }

    proptest! {
        #[test]
        fn digi_channel_conversion(dcp in any::<DigiChannelPointer>()) {
            let cp = ChannelPointer::from(&dcp);
            let dcp2 = DigiChannelPointer::try_from(&cp).unwrap();

            prop_assert_eq!(dcp, dcp2);
        }
    }
}
