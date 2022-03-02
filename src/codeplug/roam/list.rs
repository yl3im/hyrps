use std::io::Read;

use anyhow::{bail, Context};

use crate::codeplug::{
    channel_pointer::{
        digi_chan_pointer::DigiChannelPointer, raw_pointer::RawPointer,
        section::ChannelPointerSection,
    },
    cp_data::{CPData, RawCPData},
    Codeplug,
};

pub struct RoamList {
    pub channels: Vec<DigiChannelPointer>,
}

const DATA_SZ: u32 = 0x40;

impl RawCPData for RoamList {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let cps = ChannelPointerSection::load(reader)
            .context("Could not load channel pointer section")?;

        assert_eq!(cps.header.data_sz, DATA_SZ);

        let channels = cps.deduce_channels().context("Could not deduce channels")?;

        Ok(RoamList { channels })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let cps = ChannelPointerSection::from_channels(&self.channels, DATA_SZ)?;

        cps.store(writer)
    }
}

impl RoamList {
    pub fn new(channels: &[DigiChannelPointer]) -> Self {
        let mut ret = vec![DigiChannelPointer::Selected];

        ret.extend_from_slice(channels);

        Self { channels: ret }
    }
}

impl CPData for RoamList {
    fn cp_section() -> u16 {
        0x79
    }

    fn verify(&self, codeplug: &Codeplug) -> anyhow::Result<()> {
        if self.channels[0] != DigiChannelPointer::Selected {
            bail!("First channel of roam list is not <Selected>");
        }

        self.channels.iter().try_for_each(|x| x.verify(codeplug))
    }
}
