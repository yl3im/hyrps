use super::channel_pointer::pointer::ChannelPointer;
use super::cp_data::{CPData, RawCPData};
use super::disp_tabular::DisplayTabular;
use super::zone_list::ZoneList;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryInto;
use std::io::{Cursor, Read};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Zone {
    #[cfg_attr(test, proptest(regex = "[^\u{0}]{0,8}"))]
    pub name: String,
    pub no_channels: u16,
    pointer_data: [u8; 6],
}

impl Zone {
    pub fn new(name: String, channels: &Vec<ChannelPointer>) -> Self {
        let mut buf = vec![];
        let mut cursor = Cursor::new(&mut buf);

        channels[0].store(&mut cursor).unwrap();
        channels[1].store(&mut cursor).unwrap();

        buf.resize(6, 0);

        Zone {
            name,
            no_channels: channels.len() as u16,
            pointer_data: buf.try_into().unwrap(),
        }
    }
}

impl RawCPData for Zone {
    fn load(reader: &mut impl Read) -> anyhow::Result<Zone> {
        let name = String::load(reader)?;
        let no_channels = reader.read_u16::<LittleEndian>()?;
        let mut pointer_data: [u8; 6] = [0; 6];

        reader.read_exact(&mut pointer_data)?;

        Ok(Zone {
            name,
            no_channels,
            pointer_data,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        self.name.store(writer)?;

        writer.write_u16::<LittleEndian>(self.no_channels)?;

        self.pointer_data
            .iter()
            .try_for_each(|x| writer.write_u8(*x))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn zone_serde(zone in any::<super::Zone>()) {
            check_serde(&zone)?;
        }
    }
}

impl CPData for Zone {
    fn cp_section() -> u16 {
        0x24
    }
}

impl DisplayTabular for (&Zone, &ZoneList) {
    fn get_heading() -> Vec<String> {
        ["Name", "Channel(S)"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_row(&self, codeplug: &super::Codeplug) -> Vec<String> {
        let chan_names: String = itertools::intersperse(self
            .1
            .channels
            .iter()
            .map(|m| m.get_chan_name(codeplug)), "\n".to_string())
            .collect();

        vec![self.0.name.clone(), chan_names]
    }
}
