use anyhow::Context;

use crate::codeplug::channel_pointer::section::ChannelPointerSection;

use super::{
    channel_pointer::{pointer::ChannelPointer, raw_pointer::RawPointer},
    cp_data::{CPData, RawCPData},
};

use std::io::Read;

const DATA_SZ: u32 = 0x800;

pub struct ZoneList {
    pub channels: Vec<ChannelPointer>,
}

impl RawCPData for ZoneList {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let cps = ChannelPointerSection::load(reader)
            .context("Could not create channel pointer section for zone list")?;
        let channels = cps
            .deduce_channels()
            .context("Could not deduce channels for zone list section")?;

        Ok(ZoneList { channels })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        assert!(!self.channels.is_empty());

        let cps = ChannelPointerSection::from_channels(&self.channels, DATA_SZ).unwrap();

        cps.store(writer).unwrap();

        Ok(())
    }
}

impl CPData for ZoneList {
    fn cp_section() -> u16 {
        0x23
    }

    fn verify(&self, codeplug: &super::Codeplug) -> anyhow::Result<()> {
        self.channels.iter().try_for_each(|cp| cp.verify(codeplug))
    }
}
