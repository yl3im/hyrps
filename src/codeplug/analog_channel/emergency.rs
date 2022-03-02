use super::super::cp_data::RawCPData;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct EmergencySystem {
    pub idx: u8,
    pub alarm_indication: bool,
    pub alarm_ack: bool,
    pub call_indication: bool,
}

impl RawCPData for EmergencySystem {
    fn load(reader: &mut impl Read) -> anyhow::Result<EmergencySystem> {
        let idx = reader.read_u8()?;

        let b = reader.read_u8()?;

        let alarm_indication = (b & 1) != 0;
        let alarm_ack = (b & 2) != 0;
        let call_indication = (b & 4) != 0;

        Ok(EmergencySystem {
            idx,
            alarm_indication,
            alarm_ack,
            call_indication,
        })
    }

    fn store(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_u8(self.idx)?;

        let b1: u8 = self.alarm_indication as u8
            | (self.alarm_ack as u8) << 1
            | (self.call_indication as u8) << 2;

        writer.write_u8(b1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn emerg_serial_deserialise(e in any::<super::EmergencySystem>()) {
            check_serde(&e)?;
        }
    }
}
