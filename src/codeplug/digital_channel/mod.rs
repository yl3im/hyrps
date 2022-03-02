use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::codeplug::digital_channel::rrs_revert_ch::RrsRevertCh;

use self::{slrl_pointer::SLRLPointer, timeslot::Timeslot};

use super::{
    channel_common::{self, power_level::PowerLevel, ChannelCommon},
    contact::Contact,
    cp_data::{CPData, RawCPData},
    disp_tabular::DisplayTabular,
    Codeplug, CodeplugSection,
};
use std::{convert::TryFrom, io::Read};

#[cfg(test)]
use proptest_derive::Arbitrary;

pub mod rrs_revert_ch;
pub mod slrl_pointer;
pub mod timeslot;
pub mod tx_admit;

impl SLRLPointer {
    fn get_idx(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::ScanList(i) => i + 1,
            Self::RoamList(i) => i + 1,
        }
    }

    fn get_type(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::ScanList(_) => 0x10,
            Self::RoamList(_) => 0x20,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct DigitalChannel {
    pub common: channel_common::ChannelCommon,
    pub tx_admit: tx_admit::TxAdmit,
    pub tx_timeout: u8,
    pub tx_timeout_prealert: u8,
    pub tx_timeout_rekey: u8,
    pub tx_timeout_reset: u8,
    #[cfg_attr(test, proptest(strategy = "0..15u8"))]
    pub colour_code: u8,
    pub priority_interrupt_encode: bool,
    pub priority_interrupt_decode: bool,
    pub tx_contact_idx: u16,
    pub rx_group_list_idx: u16,
    pub emergency_system_idx: u16,
    pub timeslot: Timeslot,
    pub slrl_pointer: SLRLPointer,
    pub vox: bool,
    pub has_option_board: bool,
    pub loc_rev_channel_idx: u16,
    pub phone_system_idx: u16,
    pub pseudo_trunk_tx: u8,
    pub rrs_revert_ch: RrsRevertCh,
    pub auto_start_scan: bool,
    pub auto_start_roam: bool,
    pub ip_multi_site_connect: bool,
}

impl DigitalChannel {
    pub fn resolve_tx_contact<'a>(
        &self,
        contacts: &'a CodeplugSection<Contact>,
    ) -> Option<&'a Contact> {
        if self.tx_contact_idx == 0 {
            None
        } else {
            Some(&contacts.data[contacts.sec.rev_map(self.tx_contact_idx - 1)?])
        }
    }

    pub fn new(
        name: String,
        tx_freq: u32,
        rx_freq: u32,
        rx_only: bool,
        power_level: PowerLevel,
        colour_code: u8,
        tx_contact_idx: u16,
        timeslot: Timeslot,
    ) -> DigitalChannel {
        DigitalChannel {
            common: ChannelCommon {
                name,
                chan_type: channel_common::channel_type::ChannelType::Digial,
                rx_only,
                power_level,
                rx_freq,
                tx_freq,
            },
            tx_admit: tx_admit::TxAdmit::Channel,
            tx_timeout: 0,
            tx_timeout_prealert: 0,
            tx_timeout_rekey: 0,
            tx_timeout_reset: 0,
            colour_code,
            priority_interrupt_encode: false,
            priority_interrupt_decode: false,
            tx_contact_idx: tx_contact_idx + 1,
            rx_group_list_idx: 0,
            emergency_system_idx: 0,
            timeslot,
            vox: false,
            slrl_pointer: SLRLPointer::None,
            has_option_board: false,
            loc_rev_channel_idx: 0,
            phone_system_idx: 0,
            pseudo_trunk_tx: 0,
            rrs_revert_ch: RrsRevertCh::None,
            auto_start_scan: false,
            auto_start_roam: false,
            ip_multi_site_connect: false,
        }
    }
}

impl RawCPData for DigitalChannel {
    fn load(reader: &mut impl Read) -> anyhow::Result<DigitalChannel> {
        let common = channel_common::ChannelCommon::load(reader)?;

        #[cfg(not(test))]
        assert_eq!(
            common.chan_type,
            channel_common::channel_type::ChannelType::Digial
        );

        let tx_admit = tx_admit::TxAdmit::try_from(reader.read_u8()?)?;

        let tx_timeout = reader.read_u8()?;
        let tx_timeout_prealert = reader.read_u8()?;
        let tx_timeout_rekey = reader.read_u8()?;
        let tx_timeout_reset = reader.read_u8()?;

        let b2 = reader.read_u8()?;

        let colour_code = b2 & 0x0f;
        let priority_interrupt_encode = (b2 & 0x20) != 0;
        let priority_interrupt_decode = (b2 & 0x40) != 0;

        let tx_contact_idx = reader.read_u16::<LittleEndian>()?;
        let rx_group_list_idx = reader.read_u16::<LittleEndian>()?;
        let emergency_system_idx = reader.read_u16::<LittleEndian>()?;
        let slrl_idx = reader.read_u8()?;

        let b2 = reader.read_u8()?;

        assert_eq!(b2 & 0x80, 0x80);
        assert_eq!(
            (b2 & 0x40) >> 6,
            if common.rx_freq == common.tx_freq {
                0
            } else {
                1
            }
        );

        let auto_start_scan = b2 & 0x1 != 0;
        let ip_multi_site_connect = b2 & 0x20 != 0;

        let b3 = reader.read_u8()?;

        let timeslot = Timeslot::from(b3 & 0x3);
        let auto_start_roam = (b3 & 0x4) != 0;
        let slrl_type = b3 & 0x30;
        let vox = (b3 & 0x40) != 0;
        let has_option_board = (b3 & 0x80) != 0;

        let slrl_pointer = match slrl_type {
            0 => SLRLPointer::None,
            0x10 => SLRLPointer::ScanList(slrl_idx - 1),
            0x20 => SLRLPointer::RoamList(slrl_idx - 1),
            _ => bail!("Unknown SLRL pointer type: {}", slrl_type),
        };

        assert_eq!(
            reader.read_u8()?,
            if common.rx_freq == common.tx_freq {
                0
            } else {
                1
            }
        );
        assert_eq!(reader.read_u16::<LittleEndian>()?, 0);

        // Unsure about this byte. It seems to be 0xff most of the time, but
        // I've seen values of 75 and 0:
        //
        // dPMR CH 32 255
        // EL TG 9 S2 Local 0
        // EL 235 UK Call 75
        // EL TG 80 UK UA 255
        reader.read_u8()?;

        let loc_rev_channel_idx = reader.read_u16::<LittleEndian>()?;

        assert_eq!(reader.read_u16::<LittleEndian>()?, 1);

        let phone_system_idx = reader.read_u16::<LittleEndian>()?;
        let pseudo_trunk_tx = reader.read_u8()?;

        assert_eq!(reader.read_u16::<LittleEndian>()?, 0);

        let rrs_revert_ch = RrsRevertCh::from(reader.read_u16::<LittleEndian>()?);

        Ok(DigitalChannel {
            common,
            tx_admit,
            tx_timeout,
            tx_timeout_prealert,
            tx_timeout_rekey,
            tx_timeout_reset,
            colour_code,
            priority_interrupt_encode,
            priority_interrupt_decode,
            tx_contact_idx,
            rx_group_list_idx,
            emergency_system_idx,
            timeslot,
            slrl_pointer,
            vox,
            has_option_board,
            loc_rev_channel_idx,
            phone_system_idx,
            pseudo_trunk_tx,
            rrs_revert_ch,
            auto_start_scan,
            auto_start_roam,
            ip_multi_site_connect,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let direct_or_repeater_mode: u8 = if self.common.rx_freq == self.common.tx_freq {
            0
        } else {
            1
        };

        self.common.store(writer)?;

        writer.write_u8(self.tx_admit as u8)?;
        writer.write_u8(self.tx_timeout)?;
        writer.write_u8(self.tx_timeout_prealert)?;
        writer.write_u8(self.tx_timeout_rekey)?;
        writer.write_u8(self.tx_timeout_reset)?;

        writer.write_u8(
            self.colour_code
                | (self.priority_interrupt_encode as u8) << 5
                | (self.priority_interrupt_decode as u8) << 6,
        )?;

        writer.write_u16::<LittleEndian>(self.tx_contact_idx)?;
        writer.write_u16::<LittleEndian>(self.rx_group_list_idx)?;
        writer.write_u16::<LittleEndian>(self.emergency_system_idx)?;
        writer.write_u8(self.slrl_pointer.get_idx())?;

        writer.write_u8(
            0x80 | (direct_or_repeater_mode << 6)
                | (self.ip_multi_site_connect as u8) << 5
                | self.auto_start_scan as u8,
        )?;

        writer.write_u8(
            u8::from(self.timeslot)
                | self.slrl_pointer.get_type()
                | (self.auto_start_roam as u8) << 2
                | (self.vox as u8) << 6
                | (self.has_option_board as u8) << 7,
        )?;

        writer.write_u8(direct_or_repeater_mode)?;

        writer.write_u16::<LittleEndian>(0)?;

        writer.write_u8(0)?;

        writer.write_u16::<LittleEndian>(self.loc_rev_channel_idx as u16)?;

        writer.write_u16::<LittleEndian>(1)?;

        writer.write_u16::<LittleEndian>(self.phone_system_idx)?;
        writer.write_u8(self.pseudo_trunk_tx)?;

        writer.write_u16::<LittleEndian>(0)?;

        writer.write_u16::<LittleEndian>(u16::from(self.rrs_revert_ch))?;

        Ok(())
    }
}

impl CPData for DigitalChannel {
    fn cp_section() -> u16 {
        0x26
    }
}

impl DisplayTabular for DigitalChannel {
    fn get_heading() -> Vec<String> {
        let mut headings = ChannelCommon::get_heading();
        let mut dc_headings = ["TX Contact", "Colour Code", "Scan List", "Timeslot", "Vox"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        headings.append(&mut dc_headings);

        headings
    }

    fn get_row(&self, codeplug: &Codeplug) -> Vec<String> {
        let mut row = self.common.get_row(codeplug);

        row.append(&mut vec![
            match self.resolve_tx_contact(&codeplug.contacts) {
                Some(c) => c.name.clone(),
                _ => "<None>".to_string(),
            },
            self.colour_code.to_string(),
            match self.slrl_pointer {
                SLRLPointer::None => "<None>".to_string(),
                SLRLPointer::ScanList(i) => {
                    format!("SL {}", codeplug.scan_list.data.data[i as usize].name)
                }
                SLRLPointer::RoamList(i) => {
                    format!("RL {}", codeplug.roam_list.data.data[i as usize].name)
                }
            },
            format!("{:?}", self.timeslot),
            format!("{:?}", self.vox),
        ]);

        row
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn digital_channel_serde(ac in any::<super::DigitalChannel>()) {
            check_serde(&ac)?;
        }
    }
}
