use super::{
    cp_data::{CPData, RawCPData},
    disp_tabular::DisplayTabular,
    Codeplug,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{convert::TryFrom, fmt::Debug, io::Read};

#[cfg(test)]
use proptest_derive::Arbitrary;

pub mod call_type;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Contact {
    pub unk1: u16,
    pub unk2: u16,
    #[cfg_attr(test, proptest(regex = "[^\u{0}]{0,8}"))]
    pub name: String,
    pub call_type: call_type::CallType,
    pub is_ref: bool,
    pub id: u32,
}

impl Contact {
    pub fn new(name: String, call_type: call_type::CallType, id: u32) -> Contact {
        Contact {
            unk1: 0,
            unk2: 0,
            name,
            call_type,
            is_ref: true,
            id,
        }
    }
}

impl RawCPData for Contact {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let unk1 = reader.read_u16::<LittleEndian>()?;
        let unk2 = reader.read_u16::<LittleEndian>()?;
        let name = String::load(reader)?;
        let call_type = call_type::CallType::try_from(reader.read_u8()?)?;
        let is_ref = reader.read_u8()? != 0;
        assert_eq!(reader.read_u16::<LittleEndian>()?, 0);
        let id = reader.read_u32::<LittleEndian>()?;
        assert_eq!(reader.read_u32::<LittleEndian>()?, 0);

        Ok(Contact {
            unk1,
            unk2,
            name,
            call_type,
            is_ref,
            id,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        writer.write_u16::<LittleEndian>(self.unk1)?;
        writer.write_u16::<LittleEndian>(self.unk2)?;

        self.name.store(writer)?;

        writer.write_u8(self.call_type as u8)?;
        writer.write_u8(self.is_ref as u8)?;
        writer.write_u16::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(self.id)?;
        writer.write_u32::<LittleEndian>(0)?;

        Ok(())
    }
}

impl CPData for Contact {
    fn cp_section() -> u16 {
        0x2a
    }
}

impl DisplayTabular for Contact {
    fn get_heading() -> Vec<String> {
        ["Name", "Type", "ID", "Unk1", "Unk2"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_row(&self, _codeplug: &Codeplug) -> Vec<String> {
        vec![
            (*self.name).to_string(),
            format!("{:?}", self.call_type),
            self.id.to_string(),
            format!("{}", self.unk1),
            format!("{}", self.unk2),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_serde;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn contact_serde(contact in any::<super::Contact>()) {
            check_serde(&contact)?;
        }
    }
}
