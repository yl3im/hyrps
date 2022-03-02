use super::disp_tabular::DisplayTabular;
use super::Codeplug;
use super::{channel_common::channel_type::ChannelType, cp_data::RawCPData};
#[cfg(test)]
use crate::tests::check_serde;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;
use std::convert::TryFrom;
use std::io::{Read, Write};

pub mod channel_type;
pub mod power_level;

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Analog
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ChannelCommon {
    #[cfg_attr(test, proptest(regex = "[^\u{0}]{0,8}"))]
    pub name: String,
    pub chan_type: ChannelType,
    pub rx_only: bool,
    pub power_level: power_level::PowerLevel,
    pub rx_freq: u32,
    pub tx_freq: u32,
}

impl RawCPData for ChannelCommon {
    fn load(reader: &mut impl Read) -> anyhow::Result<ChannelCommon> {
        let name = String::load(reader)?;

        let chan_type = ChannelType::try_from(reader.read_u8()?)?;

        let b1 = reader.read_u8()?;
        let rx_only = (b1 & 0x1) != 0;
        let power_level = power_level::PowerLevel::from(b1);

        assert_eq!(reader.read_u16::<LittleEndian>()?, 0);

        let rx_freq = reader.read_u32::<LittleEndian>()?;
        let tx_freq = reader.read_u32::<LittleEndian>()?;

        Ok(ChannelCommon {
            name,
            chan_type,
            rx_only,
            power_level,
            rx_freq,
            tx_freq,
        })
    }

    fn store(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        self.name.store(writer)?;

        writer.write_u8(self.chan_type as u8)?;

        writer.write_u8((self.rx_only as u8) | ((self.power_level as u8) << 2))?;

        writer.write_u16::<LittleEndian>(0)?;

        writer.write_u32::<LittleEndian>(self.rx_freq)?;
        writer.write_u32::<LittleEndian>(self.tx_freq)?;

        Ok(())
    }
}

impl DisplayTabular for ChannelCommon {
    fn get_heading() -> Vec<String> {
        ["Name", "TX Freq", "RX Freq", "Power", "RX Only"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_row(&self, _codeplug: &Codeplug) -> Vec<String> {
        let freq_xform = |f: u32| (f as f64 / 1000000.0).to_string();

        vec![
            self.name.to_string(),
            freq_xform(self.tx_freq),
            freq_xform(self.rx_freq),
            format!("{:?}", self.power_level),
            format!("{:?}", self.rx_only),
        ]
    }
}

#[cfg(test)]
proptest! {
    #[test]
    fn channel_common_serde(channel in any::<ChannelCommon>()) {
        check_serde(&channel)?;
    }
}
