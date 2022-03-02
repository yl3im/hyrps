use super::{
    channel_pointer::pointer::ChannelPointer,
    section::{Section, Sections},
    Codeplug, CodeplugSection,
};
use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Error, ErrorKind, Read, Write};

pub trait RawCPData: Sized {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self>;
    fn store(&self, writer: &mut impl Write) -> anyhow::Result<()>;
}

pub trait CPData: RawCPData {
    fn cp_section() -> u16;

    fn fetch_mappings(
        &self,
        _sections: &Sections,
        _idx: u16,
    ) -> Result<Vec<ChannelPointer>, Error> {
        Ok(Vec::new())
    }

    fn get_section(sections: &Sections) -> Result<&Section, Error> {
        sections
            .get(&Self::cp_section())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "No section found"))
    }

    fn fetch_elm(sections: &Sections, idx: u16) -> anyhow::Result<Self> {
        let sec = Self::get_section(sections)?;
        let data = sec.get_data_chunk(idx)?;
        let mut cursor = Cursor::new(&data);
        Self::load(&mut cursor)
    }

    fn verify(&self, _codeplug: &Codeplug) -> anyhow::Result<()> {
        Ok(())
    }

    fn fetch_section(sections: &Sections) -> anyhow::Result<CodeplugSection<Self>> {
        let sec = Self::get_section(sections)?;

        let data = (0..sec.header.elements_in_use)
            .map(|n| Self::fetch_elm(sections, n))
            .collect::<anyhow::Result<Vec<Self>>>()?;

        Ok(CodeplugSection {
            sec: sec.clone(),
            data,
        })
    }
}

impl RawCPData for String {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf: [u16; 16] = [0; 16];

        reader.read_u16_into::<LittleEndian>(&mut buf)?;

        let p = buf
            .split_at(buf.iter().position(|v| *v == 0x0).unwrap_or(buf.len()))
            .0;

        Ok(String::from_utf16(p).map_err(|e| Error::new(ErrorKind::InvalidData, e))?)
    }

    fn store(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        let mut s: Vec<u16> = self.encode_utf16().collect();

        if s.len() > 16 {
            bail!("String '{self}' is too long");
        }

        s.resize(16, 0);

        s.iter()
            .try_for_each(|x| writer.write_u16::<LittleEndian>(*x))?;

        Ok(())
    }
}
