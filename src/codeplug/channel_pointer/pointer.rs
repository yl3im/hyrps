use anyhow::{anyhow, bail};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::io::Read;

use crate::codeplug::{cp_data::RawCPData, Codeplug};

use super::{digi_chan_pointer::DigiChannelPointer, raw_pointer::RawPointer};

#[cfg(test)]
use proptest::strategy::Strategy;

#[cfg(test)]
fn idx_strategy() -> impl Strategy<Value = u16> {
    0..(u16::MAX - 1)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ChannelPointer {
    Selected,
    #[cfg_attr(
        test,
        proptest(strategy = "idx_strategy().prop_map(ChannelPointer::Digital)")
    )]
    Digital(u16),
    #[cfg_attr(
        test,
        proptest(strategy = "idx_strategy().prop_map(ChannelPointer::Analog)")
    )]
    Analog(u16),
}

impl RawCPData for ChannelPointer {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let idx = reader.read_u16::<LittleEndian>()?;
        let target = reader.read_u8()?;
        let _flags = reader.read_u8()?;

        if idx == 0xffff {
            return Ok(ChannelPointer::Selected);
        }

        match target {
            0 => Ok(ChannelPointer::Digital(idx - 1)),
            1 => Ok(ChannelPointer::Analog(idx - 1)),
            _ => bail!("Unknown channel pointer target: {}", target),
        }
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let idx = match self {
            Self::Selected => 0xffff,
            Self::Digital(i) => i + 1,
            Self::Analog(i) => i + 1,
        };

        let target = match self {
            Self::Selected => 0,
            Self::Digital(_) => 0,
            Self::Analog(_) => 1,
        };

        writer.write_u16::<LittleEndian>(idx)?;
        writer.write_u8(target)?;
        writer.write_u8(0)?;

        Ok(())
    }
}

impl ChannelPointer {
    pub fn get_chan_name(&self, codeplug: &Codeplug) -> String {
        match self {
            Self::Selected => "<Selected>".to_string(),
            Self::Digital(i) => codeplug.digi_chans.data[*i as usize].common.name.clone(),
            Self::Analog(i) => codeplug.ana_chans.data[*i as usize].common.name.clone(),
        }
    }
}

impl From<&DigiChannelPointer> for ChannelPointer {
    fn from(value: &DigiChannelPointer) -> Self {
        match value {
            DigiChannelPointer::Selected => ChannelPointer::Selected,
            DigiChannelPointer::Digital(i) => ChannelPointer::Digital(*i),
        }
    }
}

impl RawPointer for ChannelPointer {
    fn sz() -> usize {
        0x04
    }

    fn verify(&self, cp: &Codeplug) -> anyhow::Result<()> {
        match self {
            Self::Selected => Ok(()),
            Self::Digital(i) => cp
                .digi_chans
                .data
                .get(*i as usize)
                .ok_or_else(|| anyhow!("Invalid {:?}", self))
                .map(|_| ()),
            Self::Analog(i) => cp
                .digi_chans
                .data
                .get(*i as usize)
                .ok_or_else(|| anyhow!("Invalid {:?}", self))
                .map(|_| ()),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn channel_pointer_serde(cp in any::<super::ChannelPointer>()) {
            check_serde(&cp)?;
        }
    }
}
