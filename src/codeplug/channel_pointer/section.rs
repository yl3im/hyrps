use crate::codeplug::{section::SectionMappings};

use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::io::{Cursor, Read, Seek};

use super::raw_pointer::RawPointer;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ChannelPointerSectionHeader {
    unk1: u16,
    capacity: u16,
    pub no_channels: u16,
    pub data_sz: u32,
    mappings_offset: u32,
}

impl ChannelPointerSectionHeader {
    fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let unk1 = reader.read_u16::<LittleEndian>()?;
        let capacity = reader.read_u16::<LittleEndian>()?;

        let no_channels = reader.read_u16::<LittleEndian>()?;
        let data_sz = reader.read_u32::<LittleEndian>()?;
        let mappings_offset = reader.read_u32::<LittleEndian>()?;

        Ok(ChannelPointerSectionHeader {
            unk1,
            capacity,
            no_channels,
            data_sz,
            mappings_offset,
        })
    }

    fn store(&self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        writer.write_u16::<LittleEndian>(self.unk1)?;
        writer.write_u16::<LittleEndian>(self.capacity)?;
        writer.write_u16::<LittleEndian>(self.no_channels)?;
        writer.write_u32::<LittleEndian>(self.data_sz)?;
        writer.write_u32::<LittleEndian>(self.mappings_offset)?;

        Ok(())
    }

    fn sz() -> usize {
        0xe
    }
}

pub struct ChannelPointerSection {
    pub header: ChannelPointerSectionHeader,
    data: Vec<u8>,
    mappings: Vec<SectionMappings>,
}

impl ChannelPointerSection {
    pub fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let header = ChannelPointerSectionHeader::load(reader)
            .context("Could not read channel pointer section header")?;
        let mut data = vec![0; header.data_sz as usize];
        reader
            .read_exact(&mut data)
            .context("Failed to read pointer data")?;
        let mut mappings = vec![];
        for _ in 0..header.no_channels {
            mappings.push(
                SectionMappings::load(reader).context("Failed to read pointer section mapping")?,
            );
        }

        Ok(ChannelPointerSection {
            header,
            data,
            mappings,
        })
    }

    pub fn deduce_channels<T: RawPointer>(&self) -> anyhow::Result<Vec<T>> {
        let mut ret = vec![];
        let mut cursor = Cursor::new(&self.data);

        for i in 0..self.header.no_channels {
            let data_offset = self.mappings[i as usize].offset;
            cursor.seek(std::io::SeekFrom::Start(data_offset as u64))?;

            ret.push(T::load(&mut cursor)?);
        }

        Ok(ret)
    }

    pub fn from_channels<T: RawPointer>(channels: &Vec<T>, data_sz: u32) -> Result<Self> {
        let header = ChannelPointerSectionHeader {
            unk1: 0,
            capacity: data_sz as u16 / T::sz() as u16,
            no_channels: channels.len() as u16,
            data_sz,
            mappings_offset: data_sz + ChannelPointerSectionHeader::sz() as u32,
        };

        let mut data = vec![];
        let mut mappings = vec![];
        let mut cursor = Cursor::new(&mut data);

        if header.no_channels > header.capacity {
            bail!(
                "Too many channels, max is {} but tried to add {} channels",
                header.capacity,
                header.no_channels
            );
        }

        for (idx, channel) in channels.iter().enumerate() {
            channel.store(&mut cursor)?;
            mappings.push(SectionMappings {
                idx: idx as u16,
                offset: idx as u32 * T::sz() as u32,
            })
        }

        data.resize(data_sz as usize, 0);

        Ok(ChannelPointerSection {
            header,
            data,
            mappings,
        })
    }

    pub fn store(&self, writer: &mut impl std::io::Write) -> Result<()> {
        self.header.store(writer)?;
        writer.write_all(&self.data)?;
        self.mappings.iter().try_for_each(|x| x.write(writer))?;

        Ok(())
    }
}
