use super::{
    channel_pointer::pointer::ChannelPointer,
    cp_data::{CPData, RawCPData},
    disp_tabular::DisplayTabular,
    scan_list::ScanList,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{convert::TryFrom, io::Read};

#[cfg(test)]
use proptest_derive::Arbitrary;

mod ctcss_scan_mode;
mod scan_type;
mod tx_mode;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Scan {
    #[cfg_attr(test, proptest(regex = "[^\u{0}]{0,8}"))]
    pub name: String,
    pub scan_type: scan_type::ScanType,
    pub ctcss_mode: ctcss_scan_mode::CTCSSScanMode,
    pub tx_mode: tx_mode::ScanTxMode,
    pub designated_tx_channel: ChannelPointer,
}

impl RawCPData for Scan {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let name = String::load(reader)?;
        let flags = reader.read_u16::<LittleEndian>()?;

        let scan_type = scan_type::ScanType::try_from(flags)?;
        let ctcss_mode = ctcss_scan_mode::CTCSSScanMode::try_from(flags)?;
        let tx_mode = tx_mode::ScanTxMode::try_from(flags)?;

        let designated_tx_channel = ChannelPointer::load(reader)?;

        Ok(Scan {
            name,
            scan_type,
            ctcss_mode,
            tx_mode,
            designated_tx_channel,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        self.name.store(writer)?;

        writer.write_u16::<LittleEndian>(
            (self.scan_type as u16) | (self.ctcss_mode as u16) << 2 | (self.tx_mode as u16) << 6,
        )?;

        self.designated_tx_channel.store(writer)?;

        writer.write_all(&[0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0x08, 0x6, 0x14, 0x06])?;

        Ok(())
    }
}

impl Scan {
    pub fn new(name: String) -> Scan {
        Scan {
            name,
            scan_type: scan_type::ScanType::Normal,
            ctcss_mode: ctcss_scan_mode::CTCSSScanMode::Disabled,
            tx_mode: tx_mode::ScanTxMode::Selected,
            designated_tx_channel: ChannelPointer::Digital(0),
        }
    }
}

impl CPData for Scan {
    fn cp_section() -> u16 {
        0x6d
    }
}

impl DisplayTabular for (&Scan, &ScanList) {
    fn get_heading() -> Vec<String> {
        [
            "Name",
            "Scan Type",
            "CTCSS Scan Mode",
            "TX Mode",
            "Channel(S)",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn get_row(&self, codeplug: &super::Codeplug) -> Vec<String> {
        assert_eq!(self.1.channels[0], ChannelPointer::Selected);

        let chan_names: String = itertools::intersperse(self
            .1
            .channels
            .iter()
            .map(|m| m.get_chan_name(codeplug)), "\n".to_string())
            .collect();

        vec![
            self.0.name.clone(),
            format!("{:?}", self.0.scan_type),
            format!("{:?}", self.0.ctcss_mode),
            format!("{:?}", self.0.tx_mode),
            chan_names,
        ]
    }
}
