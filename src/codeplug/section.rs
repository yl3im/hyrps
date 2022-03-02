use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    io::{Error, Read, Seek, SeekFrom, Write},
};

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct SectionHeader {
    pub section_type: u16,
    #[cfg_attr(test, proptest(strategy = "0..4096u16"))]
    pub capacity: u16,
    #[cfg_attr(test, proptest(strategy = "0..15u8"))]
    pub unk1: u8,
    pub elements_in_use: u16,
    pub unk2: u32,
    pub byte_size: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct SectionMappings {
    pub idx: u16,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub header: SectionHeader,
    pub data: Vec<u8>,
    pub mappings: Vec<SectionMappings>,
    pub addr: u64,
}

impl SectionHeader {
    pub fn load(reader: &mut impl Read) -> Result<Self, Error> {
        let section_type = reader.read_u16::<LittleEndian>()?;
        let cap_field = reader.read_u16::<LittleEndian>()?;
        let capacity = cap_field & 0xfff;
        let unk1 = (cap_field >> 12) as u8;
        let elements_in_use = reader.read_u16::<LittleEndian>()?;
        assert_eq!(reader.read_u32::<LittleEndian>()?, 0x00000020);
        let unk2 = reader.read_u32::<LittleEndian>()?;
        let byte_size = reader.read_u32::<LittleEndian>()?;
        assert_eq!(reader.read_u32::<LittleEndian>()?, byte_size + 0x16);

        Ok(SectionHeader {
            section_type,
            capacity,
            unk1,
            elements_in_use,
            unk2,
            byte_size,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        writer.write_u16::<LittleEndian>(self.section_type)?;
        let cap_field = self.capacity | (self.unk1 as u16) << 12;
        writer.write_u16::<LittleEndian>(cap_field)?;
        writer.write_u16::<LittleEndian>(self.elements_in_use)?;
        writer.write_u32::<LittleEndian>(0x00000020)?;
        writer.write_u32::<LittleEndian>(self.unk2)?;
        writer.write_u32::<LittleEndian>(self.byte_size)?;
        writer.write_u32::<LittleEndian>(self.byte_size + 0x16)?;

        Ok(())
    }
}

impl SectionMappings {
    pub fn load(reader: &mut impl Read) -> Result<Self, Error> {
        let idx = reader.read_u16::<LittleEndian>()?;
        let offset = reader.read_u32::<LittleEndian>()?;

        Ok(SectionMappings { idx, offset })
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        writer.write_u16::<LittleEndian>(self.idx)?;
        writer.write_u32::<LittleEndian>(self.offset)?;

        Ok(())
    }
}

pub type Sections = HashMap<u16, Section>;

impl Section {
    fn load(reader: &mut (impl Read + Seek)) -> Result<Self, Error> {
        let addr = reader.stream_position()?;
        let header = SectionHeader::load(reader)?;
        let mut data = vec![0; header.byte_size as usize];
        reader.read_exact(&mut data)?;
        let mut mappings = Vec::new();

        for _ in 0..header.capacity {
            mappings.push(SectionMappings::load(reader)?);
        }

        Ok(Section {
            header,
            data,
            mappings,
            addr,
        })
    }

    pub fn get_mapping(&self, idx: u16) -> SectionMappings {
        SectionMappings {
            idx,
            offset: (idx as usize * self.get_element_sz()) as u32,
        }
    }

    pub fn rev_map(&self, idx: u16) -> Option<usize> {
        self.mappings.iter().position(|m| m.idx == idx)
    }

    pub fn get_element_sz(&self) -> usize {
        assert!((self.header.byte_size % (self.header.capacity as u32)) == 0);

        (self.header.byte_size / self.header.capacity as u32) as usize
    }

    pub fn get_data_chunk(&self, idx: u16) -> Result<&[u8], Error> {
        let mut data_chunks = self.data.chunks(self.get_element_sz());
        let data_idx = self.mappings[idx as usize].idx;
        data_chunks.nth(data_idx as usize).ok_or_else(|| Error::new(
            std::io::ErrorKind::InvalidData,
            "Could not find index for data",
        ))
    }

    pub fn load_sections(data: &mut (impl Read + Seek)) -> Result<Sections, Error> {
        let addr = 0x38e;

        data.seek(SeekFrom::Start(addr))?;

        let end_addr = u64::from(data.read_u32::<LittleEndian>()?);
        let mut sections = HashMap::new();

        while data.stream_position()? < end_addr {
            let section = Section::load(data)?;
            sections.insert(section.header.section_type, section);
        }

        Ok(sections)
    }
}

impl Display for Section {
    fn fmt(&self, w: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(w,
            "Section addr=0x{:06X} type=0x{:04X} capacity=0x{:04X} elements_in_use=0x{:04X} byte_size=0x{:08X} unk1=0x{:02X}",
            self.addr,
            self.header.section_type,
            self.header.capacity,
            self.header.elements_in_use,
            self.header.byte_size,
            self.header.unk1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SectionHeader;
    use proptest::prelude::*;
    use std::io::Seek;

    proptest! {
        #[test]
        fn section_header_serde(sh in any::<super::SectionHeader>()) {
            let v = Vec::new();
            let mut cursor = std::io::Cursor::new(v);

            sh.write(&mut cursor).unwrap();

            cursor.rewind().unwrap();

            let sh2 = SectionHeader::load(&mut cursor).unwrap();

            prop_assert_eq!(sh, sh2);
        }
    }

    proptest! {
        #[test]
        fn section_mappings_serde(sm in any::<super::SectionMappings>()) {
            let v = Vec::new();
            let mut cursor = std::io::Cursor::new(v);

            sm.write(&mut cursor).unwrap();

            cursor.rewind().unwrap();

            let sm2 = super::SectionMappings::load(&mut cursor).unwrap();

            prop_assert_eq!(sm, sm2);
        }
    }
}
