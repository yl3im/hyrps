use std::io::Read;

use super::{
    channel_pointer::{
        section::ChannelPointerSection,
        {pointer::ChannelPointer, raw_pointer::RawPointer},
    },
    cp_data::{CPData, RawCPData},
};

use anyhow::bail;

pub struct ScanList {
    pub channels: Vec<ChannelPointer>,
}

const DATA_SZ: u32 = 0x80;

impl RawCPData for ScanList {
    fn load(reader: &mut impl Read) -> anyhow::Result<ScanList> {
        let cps = ChannelPointerSection::load(reader)?;

        assert_eq!(cps.header.data_sz, DATA_SZ);

        let channels = cps.deduce_channels()?;

        Ok(ScanList { channels })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let cps = ChannelPointerSection::from_channels(&self.channels, DATA_SZ)?;

        cps.store(writer)?;

        Ok(())
    }
}

impl ScanList {
    pub fn new(channels: &[ChannelPointer]) -> ScanList {
        let mut ret = vec![ChannelPointer::Selected];

        ret.extend_from_slice(channels);

        ScanList { channels: ret }
    }
}

impl CPData for ScanList {
    fn cp_section() -> u16 {
        0x4d
    }

    fn verify(&self, codeplug: &super::Codeplug) -> anyhow::Result<()> {
        if self.channels[0] != ChannelPointer::Selected {
            bail!("First channel of scan list is not <Selected>");
        }

        self.channels.iter().try_for_each(|cp| cp.verify(codeplug))
    }
}
