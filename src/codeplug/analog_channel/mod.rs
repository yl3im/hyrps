use super::{
    analog_channel::{ctcss::Ctcss, signalling_type::SignallingType},
    channel_common::{self, power_level::PowerLevel, ChannelCommon},
    cp_data::{CPData, RawCPData},
    disp_tabular::DisplayTabular,
    Codeplug,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{convert::TryFrom, io::Read};

use self::{
    auto_reset_mode::AutoResetMode, carrier_sql_level::CarrierSqlLevel,
    channel_change_sql_mode::ChannelChangeSqlMode, ctcss::CTCSSType, emergency::EmergencySystem,
    sql_mode::SqlMode, tx_admit::TxAdmit,
};

#[cfg(test)]
use proptest_derive::Arbitrary;

pub mod auto_reset_mode;
pub mod carrier_sql_level;
pub mod channel_change_sql_mode;
pub mod ctcss;
pub mod ctcss_tail_revert_phase;
pub mod emergency;
pub mod signalling_type;
pub mod sql_mode;
pub mod tx_admit;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct AnalogChannel {
    pub common: channel_common::ChannelCommon,
    pub rx_ctcss: Ctcss,
    pub tx_ctcss: Ctcss,
    pub rx_sql_mode: SqlMode,
    pub mon_sql_mode: SqlMode,
    pub channel_change_sql_mode: ChannelChangeSqlMode,
    pub carrier_sql_level: CarrierSqlLevel,
    pub tx_admit: TxAdmit,
    pub tx_timeout: u8,
    pub tot_prealert: u8,
    pub tot_rekey: u8,
    pub tot_reset: u8,
    pub auto_reset_mode: AutoResetMode,
    pub auto_reset_time: u8,
    pub signalling_type: SignallingType,
    pub emergency: EmergencySystem,
    pub scan_list_idx: u8,
    pub auto_start_scan: bool,
    pub emph_de_emph: bool,
    pub scrambler: bool,
    pub compandor: bool,
    pub vox: bool,
}

fn error_xform(e: impl std::string::ToString) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

impl RawCPData for AnalogChannel {
    fn load(reader: &mut impl Read) -> anyhow::Result<AnalogChannel> {
        let common = channel_common::ChannelCommon::load(reader)?;

        #[cfg(not(test))]
        assert_eq!(
            common.chan_type,
            channel_common::channel_type::ChannelType::Analog
        );

        let rx_ctcss = Ctcss::load(reader)?;
        let tx_ctcss = Ctcss::load(reader)?;

        let rx_sql_mode = sql_mode::SqlMode::try_from(reader.read_u8()?).map_err(error_xform)?;

        let mon_sql_mode = sql_mode::SqlMode::try_from(reader.read_u8()?).map_err(error_xform)?;

        let channel_change_sql_mode =
            ChannelChangeSqlMode::try_from(reader.read_u8()?).map_err(error_xform)?;

        assert_eq!(reader.read_u8()?, 0x0);

        let carrier_sql_level =
            CarrierSqlLevel::try_from(reader.read_u8()?).map_err(error_xform)?;

        let tx_admit = TxAdmit::try_from(reader.read_u8()?).map_err(error_xform)?;

        let tx_timeout = reader.read_u8()?;
        let tot_prealert = reader.read_u8()?;
        let tot_rekey = reader.read_u8()?;
        let tot_reset = reader.read_u8()?;

        let auto_reset_mode = AutoResetMode::try_from(reader.read_u8()?).map_err(error_xform)?;

        let auto_reset_time = reader.read_u8()?;

        assert_eq!(reader.read_u8()?, 10);

        let signalling_type = SignallingType::try_from(reader.read_u8()?).map_err(error_xform)?;

        assert_eq!(reader.read_u8()?, 0);

        let emergency = EmergencySystem::load(reader)?;

        let scan_list_idx = reader.read_u8()?;

        let b1 = reader.read_u8()?;

        let auto_start_scan = (b1 & 0x1) != 0;
        let emph_de_emph = (b1 & 0x10) != 0;
        let compandor = (b1 & 0x20) != 0;
        let scrambler = (b1 & 0x40) != 0;

        let vox = (reader.read_u8()? & 0x80) != 0;

        Ok(AnalogChannel {
            common,
            rx_ctcss,
            tx_ctcss,
            rx_sql_mode,
            mon_sql_mode,
            channel_change_sql_mode,
            carrier_sql_level,
            tx_admit,
            tx_timeout,
            tot_prealert,
            tot_rekey,
            tot_reset,
            auto_reset_mode,
            auto_reset_time,
            signalling_type,
            emergency,
            scan_list_idx,
            auto_start_scan,
            emph_de_emph,
            scrambler,
            compandor,
            vox,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        self.common.store(writer)?;
        self.rx_ctcss.store(writer)?;
        self.tx_ctcss.store(writer)?;

        writer.write_u8(self.rx_sql_mode as u8)?;
        writer.write_u8(self.mon_sql_mode as u8)?;
        writer.write_u8(self.channel_change_sql_mode as u8)?;

        writer.write_u8(0)?;

        writer.write_u8(self.carrier_sql_level as u8)?;
        writer.write_u8(self.tx_admit as u8)?;

        writer.write_u8(self.tx_timeout)?;
        writer.write_u8(self.tot_prealert)?;
        writer.write_u8(self.tot_rekey)?;
        writer.write_u8(self.tot_reset)?;

        writer.write_u8(self.auto_reset_mode as u8)?;
        writer.write_u8(self.auto_reset_time)?;

        writer.write_u8(10)?;

        writer.write_u8(self.signalling_type as u8)?;

        writer.write_u8(0)?;

        self.emergency.store(writer)?;

        writer.write_u8(self.scan_list_idx)?;

        writer.write_u8(
            (self.auto_start_scan as u8)
                | (self.emph_de_emph as u8) << 4
                | (self.compandor as u8) << 5
                | (self.scrambler as u8) << 6,
        )?;

        writer.write_u8((self.vox as u8) << 7)?;

        Ok(())
    }
}

impl AnalogChannel {
    pub fn new(
        name: String,
        tx_freq: u32,
        rx_freq: u32,
        rx_only: bool,
        power_level: PowerLevel,
        rx_ctcss: Ctcss,
        tx_ctcss: Ctcss,
    ) -> AnalogChannel {
        AnalogChannel {
            common: ChannelCommon {
                name,
                chan_type: channel_common::channel_type::ChannelType::Analog,
                rx_only,
                power_level,
                rx_freq,
                tx_freq,
            },
            rx_ctcss,
            tx_ctcss,
            rx_sql_mode: SqlMode::Carrier,
            mon_sql_mode: SqlMode::Carrier,
            channel_change_sql_mode: ChannelChangeSqlMode::RxSQLMode,
            carrier_sql_level: CarrierSqlLevel::Normal,
            tx_admit: TxAdmit::ChannelFree,
            tx_timeout: 0,
            tot_prealert: 0,
            tot_rekey: 0,
            tot_reset: 0,
            auto_reset_mode: AutoResetMode::Disable,
            auto_reset_time: 0,
            signalling_type: SignallingType::None,
            emergency: EmergencySystem {
                idx: 0,
                alarm_indication: false,
                alarm_ack: false,
                call_indication: false,
            },
            scan_list_idx: 0,
            auto_start_scan: false,
            emph_de_emph: true,
            scrambler: false,
            compandor: false,
            vox: false,
        }
    }
}

impl CPData for AnalogChannel {
    fn cp_section() -> u16 {
        0x27
    }
}

impl DisplayTabular for AnalogChannel {
    fn get_heading() -> Vec<String> {
        let mut headings = ChannelCommon::get_heading();
        let mut ac_headings = ["TX CTCSS", "RX CCTCSS", "Squench Mode", "VOX"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        headings.append(&mut ac_headings);

        headings
    }

    fn get_row(&self, codeplug: &Codeplug) -> Vec<String> {
        let mut row = self.common.get_row(codeplug);

        let fmt_ctcss = |c: &Ctcss| match c.kind {
            CTCSSType::None => "<None>".to_string(),
            CTCSSType::Ctcss => format!("CTCSS ({})", c.freq),
            CTCSSType::Cdcss => format!("CDCSS ({})", c.freq),
            CTCSSType::CdcssInvert => format!("CDCSS Revert ({})", c.freq),
        };

        row.append(&mut vec![
            fmt_ctcss(&self.tx_ctcss),
            fmt_ctcss(&self.rx_ctcss),
            format!("{:?}", self.rx_sql_mode),
            format!("{}", self.vox),
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
        fn analog_channel_serde(ac in any::<super::AnalogChannel>()) {
            check_serde(&ac)?;
        }
    }
}
