use self::list::RoamList;
use super::{
    cp_data::{CPData, RawCPData},
    disp_tabular::DisplayTabular,
};
use crate::codeplug::channel_pointer::digi_chan_pointer::DigiChannelPointer;
use byteorder::{ReadBytesExt, WriteBytesExt};

#[cfg(test)]
use proptest_derive::Arbitrary;

pub mod list;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Roam {
    #[cfg_attr(test, proptest(regex = "[^\u{0}]{0,8}"))]
    pub name: String,
    pub rssi_threshold: u8,
    pub rssi_offset: u8,
    pub interval_time: u8,
    pub active_site_roam: bool,
    pub return_to_selected_ch: bool,
    pub follow_all_master_site_config: bool,
    pub stay: bool,
}

const PADDING: [u8; 3] = [0u8; 3];

impl RawCPData for Roam {
    fn load(reader: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let name = String::load(reader)?;
        let rssi_threshold = reader.read_u8()?;

        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        assert_eq!(buf, PADDING);

        let flags = reader.read_u8()?;
        let rssi_offset = reader.read_u8()?;
        let interval_time = reader.read_u8()?;

        assert_eq!(reader.read_u8()?, 0);

        Ok(Roam {
            name,
            rssi_threshold,
            rssi_offset,
            interval_time,
            active_site_roam: flags & 0x2 != 0,
            return_to_selected_ch: flags & 0x4 != 0,
            follow_all_master_site_config: flags & 0x8 != 0,
            stay: flags & 0x10 != 0,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        self.name.store(writer)?;
        writer.write_u8(self.rssi_threshold)?;

        writer.write_all(&PADDING)?;

        let flags = (self.active_site_roam as u8) << 1
            | (self.return_to_selected_ch as u8) << 2
            | (self.follow_all_master_site_config as u8) << 3
            | (self.stay as u8) << 4;

        writer.write_u8(flags)?;

        writer.write_u8(self.rssi_offset)?;
        writer.write_u8(self.interval_time)?;

        writer.write_u8(0)?;

        Ok(())
    }
}

impl Roam {
    pub fn new(name: String) -> Self {
        Roam {
            name,
            rssi_threshold: 108,
            rssi_offset: 5,
            interval_time: 15,
            active_site_roam: true,
            return_to_selected_ch: false,
            follow_all_master_site_config: false,
            stay: true,
        }
    }
}

impl CPData for Roam {
    fn cp_section() -> u16 {
        0x7a
    }
}

impl DisplayTabular for (&Roam, &RoamList) {
    fn get_heading() -> Vec<String> {
        [
            "Name",
            "RSSI Threshold",
            "RSSI Offset(dB)",
            "Interval Time(s)",
            "Channel(s)",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn get_row(&self, codeplug: &super::Codeplug) -> Vec<String> {
        assert_eq!(self.1.channels[0], DigiChannelPointer::Selected);

        let chan_names: String = itertools::intersperse(self
            .1
            .channels
            .iter()
            .map(|m| m.get_chan_name(codeplug)), "\n".to_string())
            .collect();

        vec![
            self.0.name.clone(),
            format!("-{:?}", self.0.rssi_threshold),
            format!("{:?}", self.0.rssi_offset),
            format!("{:?}", self.0.interval_time),
            chan_names,
        ]
    }
}
#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roam_serde(roam in any::<super::Roam>()) {
            check_serde(&roam)?;
        }
    }
}
