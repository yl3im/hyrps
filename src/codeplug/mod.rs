use crate::codeplug::disp_tabular::DisplayTabular;
use std::{
    fmt::Display,
    io::{Read, Seek, SeekFrom, Write},
};

use self::{
    analog_channel::AnalogChannel,
    contact::Contact,
    cp_data::CPData,
    digital_channel::DigitalChannel,
    roam::{list::RoamList, Roam},
    scan::Scan,
    scan_list::ScanList,
    section::Section,
    zone::Zone,
    zone_list::ZoneList,
};

use anyhow::{bail, Context, Result};

pub mod analog_channel;
pub mod channel_common;
pub mod channel_pointer;
pub mod contact;
pub mod cp_data;
pub mod digital_channel;
pub mod disp_tabular;
pub mod roam;
pub mod scan;
pub mod scan_list;
pub mod section;
pub mod zone;
pub mod zone_list;

pub struct CodeplugSectionWithChanList<T: CPData, M: CPData> {
    pub data: CodeplugSection<T>,
    pub channels: CodeplugSection<M>,
}

impl<T: CPData, M: CPData> CodeplugSectionWithChanList<T, M> {
    fn clear(&mut self) {
        self.data.clear();
        self.channels.clear();
    }

    pub fn insert(&mut self, obj: T, channels: M) -> usize {
        let n = self.data.insert(obj);
        assert_eq!(n, self.channels.insert(channels));

        n
    }

    fn verify(&self, cp: &Codeplug) -> Result<()> {
        if self.data.sec.header.elements_in_use != self.channels.sec.header.elements_in_use {
            bail!(
                "Number of channel lists and data elements for {} do not match",
                std::any::type_name::<T>()
            );
        }

        self.data.verify(cp)?;
        self.channels.verify(cp)
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        self.data.write(writer)?;
        self.channels.write(writer)
    }
}

pub struct CodeplugSection<T: CPData> {
    pub sec: Section,
    pub data: Vec<T>,
}

impl<T: CPData> CodeplugSection<T> {
    pub fn insert(&mut self, obj: T) -> usize {
        let n = self.data.len();
        self.data.push(obj);
        self.sec.header.elements_in_use += 1;

        assert!(self.sec.header.elements_in_use < self.sec.header.capacity);

        n
    }

    fn clear(&mut self) {
        self.sec.header.elements_in_use = 0;
        self.data.clear();
    }

    fn verify(&self, cp: &Codeplug) -> Result<()> {
        if self.sec.header.elements_in_use as usize != self.data.len() {
            bail!("Section header does not match number of data elements")
        }

        self.data.iter().try_for_each(|x| x.verify(cp))
    }

    fn write(&self, writer: &mut (impl Write + Seek)) -> Result<()> {
        let mut buf = vec![];

        for obj in self.data.iter() {
            let mut obj_buf = vec![];
            obj.store(&mut obj_buf).context("Could not store object")?;

            if obj_buf.len() != self.sec.get_element_sz() {
                println!(
                    "WARNING: Padding {} object to correct size",
                    std::any::type_name::<Self>()
                );
                obj_buf.resize(self.sec.get_element_sz(), 0);
            }

            buf.append(&mut obj_buf);
        }

        assert!(buf.len() < self.sec.header.byte_size as usize);

        buf.resize(self.sec.header.byte_size as usize, 0_u8);

        writer
            .seek(std::io::SeekFrom::Start(self.sec.addr))
            .context("Could not seek to position")?;

        self.sec
            .header
            .write(writer)
            .context("Error writing header")?;

        writer
            .write_all(&buf)
            .context("Failed to write section data")?;

        for i in 0..(self.sec.header.capacity) {
            self.sec
                .get_mapping(i)
                .write(writer)
                .context("Failed to write section mapping")?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum DeviceType {
    Portable,
    Mobile,
}

pub struct Codeplug {
    pub contacts: CodeplugSection<Contact>,
    pub zones: CodeplugSectionWithChanList<Zone, ZoneList>,
    pub digi_chans: CodeplugSection<DigitalChannel>,
    pub ana_chans: CodeplugSection<AnalogChannel>,
    pub scan_list: CodeplugSectionWithChanList<Scan, ScanList>,
    pub roam_list: CodeplugSectionWithChanList<Roam, RoamList>,
    model: String,
}

impl Codeplug {
    pub fn read_codeplug(data: &mut (impl Read + Seek)) -> Result<Self> {
        let model = Self::get_radio_model(data)?;
        let sections = Section::load_sections(data)?;
        let zones = Zone::fetch_section(&sections)?;
        let zone_list = ZoneList::fetch_section(&sections)?;
        let scan = Scan::fetch_section(&sections)?;
        let scan_list = ScanList::fetch_section(&sections)?;
        let roam = Roam::fetch_section(&sections)?;
        let roam_list =
            RoamList::fetch_section(&sections).context("Could not read roam channel list")?;

        Ok(Codeplug {
            contacts: Contact::fetch_section(&sections)?,
            digi_chans: DigitalChannel::fetch_section(&sections)?,
            ana_chans: AnalogChannel::fetch_section(&sections)?,
            zones: CodeplugSectionWithChanList {
                data: zones,
                channels: zone_list,
            },
            scan_list: CodeplugSectionWithChanList {
                data: scan,
                channels: scan_list,
            },
            roam_list: CodeplugSectionWithChanList {
                data: roam,
                channels: roam_list,
            },
            model,
        })
    }

    pub fn get_radio_model(data: &mut (impl Read + Seek)) -> Result<String> {
        let mut ret = String::new();
        data.seek(SeekFrom::Start(0x3c))
            .context("Failed to seek to model offset")?;

        data.take(27)
            .read_to_string(&mut ret)
            .context("Failed to read radio model string")?;

        Ok(ret)
    }

    pub fn radio_type(&self) -> DeviceType {
        if self.model.starts_with('m') {
            DeviceType::Mobile
        } else {
            DeviceType::Portable
        }
    }

    pub fn write_codeplug(&mut self, writer: &mut (impl Write + Seek)) -> Result<()> {
        self.verify()?;

        self.contacts
            .write(writer)
            .context("Failed to write contact section")?;

        self.digi_chans
            .write(writer)
            .context("Failed to write Digital Channel section")?;

        self.ana_chans
            .write(writer)
            .context("Failed to write analog channel section")?;

        self.zones
            .write(writer)
            .context("Failed to write zone section")?;

        self.scan_list
            .write(writer)
            .context("Failed to write scan list section")?;

        self.roam_list
            .write(writer)
            .context("Failed to write roam lists section")
    }

    pub fn verify(&self) -> anyhow::Result<()> {
        self.contacts.verify(self)?;
        self.ana_chans.verify(self)?;
        self.digi_chans.verify(self)?;
        self.scan_list.verify(self)?;
        self.zones.verify(self)
    }

    pub fn clear_codeplug(&mut self) {
        self.contacts.clear();
        self.ana_chans.clear();
        self.digi_chans.clear();
        self.scan_list.clear();
        self.zones.clear();
        self.roam_list.clear();
    }
}

impl Display for Codeplug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contacts\n")?;
        write!(f, "========\n")?;
        Contact::print_table(&self.contacts.data, self);

        write!(f, "Digital Channels\n")?;
        write!(f, "================\n")?;
        DigitalChannel::print_table(&self.digi_chans.data, self);

        write!(f, "Analog Channels\n")?;
        write!(f, "===============\n")?;
        AnalogChannel::print_table(&self.ana_chans.data, self);

        write!(f, "Scan Lists\n")?;
        write!(f, "==========\n")?;
        <(&Scan, &ScanList)>::print_table(
            &self
                .scan_list
                .data
                .data
                .iter()
                .zip(&self.scan_list.channels.data)
                .collect::<Vec<_>>(),
            self,
        );

        write!(f, "Roam Lists\n")?;
        write!(f, "==========\n")?;
        <(&Roam, &RoamList)>::print_table(
            &self
                .roam_list
                .data
                .data
                .iter()
                .zip(&self.roam_list.channels.data)
                .collect::<Vec<_>>(),
            self,
        );

        write!(f, "Zones\n")?;
        write!(f, "=====\n")?;
        <(&Zone, &ZoneList)>::print_table(
            &self
                .zones
                .data
                .data
                .iter()
                .zip(&self.zones.channels.data)
                .collect::<Vec<_>>(),
            self,
        );

        Ok(())
    }
}
