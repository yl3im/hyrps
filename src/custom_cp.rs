use std::convert::TryFrom;

use anyhow::bail;
use itertools::Itertools;

use crate::codeplug::{
    analog_channel::{
        ctcss::{CTCSSType, Ctcss},
        AnalogChannel,
    },
    channel_common::power_level::PowerLevel,
    channel_pointer::{digi_chan_pointer::DigiChannelPointer, pointer::ChannelPointer},
    contact::Contact,
    digital_channel::{slrl_pointer::SLRLPointer, timeslot::Timeslot, DigitalChannel},
    roam::{list::RoamList, Roam},
    scan::Scan,
    scan_list::ScanList,
    zone::Zone,
    zone_list::ZoneList,
    Codeplug, DeviceType,
};

impl Codeplug {
    fn create_or_add_contact(&mut self, name: &str, id: u32) -> usize {
        match self.contacts.data.iter().find_position(|x| x.id == id) {
            Some(i) => i.0,
            None => self.contacts.insert(Contact::new(
                format!("{} {}", id, name),
                crate::codeplug::contact::call_type::CallType::Group,
                id,
            )),
        }
    }

    fn set_zone_scanlist(&mut self, zone_idx: usize, scan_list_idx: usize) -> anyhow::Result<()> {
        for cptr in self.zones.channels.data[zone_idx].channels.iter() {
            match cptr {
                ChannelPointer::Digital(idx) => {
                    let mut dc = &mut self.digi_chans.data[*idx as usize];
                    dc.slrl_pointer = SLRLPointer::ScanList(scan_list_idx as u8);
                    dc.auto_start_scan = true;
                }
                ChannelPointer::Analog(idx) => {
                    let mut ac = &mut self.ana_chans.data[*idx as usize];
                    ac.scan_list_idx = scan_list_idx as u8 + 1;
                    ac.auto_start_scan = true;
                }
                ChannelPointer::Selected => bail!("Can't set scanlist of selected channel"),
            }
        }

        Ok(())
    }

    fn slot_name(ts: Timeslot) -> String {
        match ts {
            Timeslot::Slot1 => "S1".to_string(),
            Timeslot::Slot2 => "S2".to_string(),
            Timeslot::PseudoTrunk => "PS".to_string(),
        }
    }

    fn add_dmr_repeater(
        &mut self,
        suffix: &str,
        loc: &str,
        tx_freq: u32,
        rx_freq: u32,
        colour_code: u8,
        roam_862: Option<&mut Vec<DigiChannelPointer>>,
    ) -> usize {
        let mut contact_spec = vec![
            ("WW", 1, Timeslot::Slot1, false),
            ("Europe", 2, Timeslot::Slot1, false),
            ("UK Call", 235, Timeslot::Slot1, false),
            ("UK UA", 80, Timeslot::Slot1, false),
            ("UK UA", 81, Timeslot::Slot1, false),
            ("UK UA", 82, Timeslot::Slot1, false),
            ("UK UA", 83, Timeslot::Slot1, false),
            ("CQ-UK UA", 2351, Timeslot::Slot1, false),
            ("NW", 820, Timeslot::Slot1, false),
            ("Echo", 9990, Timeslot::Slot1, false),
            ("Local", 9, Timeslot::Slot1, true),
            ("Local", 9, Timeslot::Slot2, true),
        ];

        if roam_862.is_some() {
            contact_spec.push(("M62 Roam", 862, Timeslot::Slot2, false));
        }

        let digi_power_level = if self.radio_type() == DeviceType::Portable {
            PowerLevel::High
        } else {
            PowerLevel::Low
        };

        let channels = contact_spec
            .iter()
            .map(|cs| {
                let name = if cs.3 {
                    format!("{} {} {} {}", suffix, cs.1, Self::slot_name(cs.2), cs.0)
                } else {
                    format!("{} {} {}", suffix, cs.1, cs.0)
                };

                let contact_idx = self.create_or_add_contact(cs.0, cs.1);
                let idx = self.digi_chans.insert(DigitalChannel::new(
                    name,
                    tx_freq,
                    rx_freq,
                    false,
                    digi_power_level,
                    colour_code,
                    contact_idx as u16,
                    cs.2,
                ));

                ChannelPointer::Digital(idx as u16)
            })
            .collect::<Vec<_>>();

        if let Some(roam_chans) = roam_862 {
            let dcp = DigiChannelPointer::try_from(channels.last().unwrap()).unwrap();
            roam_chans.push(dcp)
        }

        let name = format!("GB7{suffix} {loc}");

        let scan = Scan::new(name.clone());
        let scan_list = ScanList::new(&channels);

        let scan_idx = self.scan_list.insert(scan, scan_list);

        let zone = Zone::new(name, &channels);
        let zone_list = ZoneList {
            channels,
        };

        let zone_idx = self.zones.insert(zone, zone_list);

        self.set_zone_scanlist(zone_idx, scan_idx).unwrap();

        zone_idx
    }

    fn add_analog_zone(
        &mut self,
        name: String,
        power_level: PowerLevel,
        i: &mut dyn Iterator<Item = (String, u32, u32)>,
    ) -> usize {
        let channels: Vec<ChannelPointer> = i
            .map(|(n, tx_freq, rx_freq)| {
                let idx = self.ana_chans.insert(AnalogChannel::new(
                    n,
                    tx_freq,
                    rx_freq,
                    false,
                    power_level,
                    Ctcss {
                        kind: CTCSSType::None,
                        freq: 0,
                    },
                    Ctcss {
                        kind: CTCSSType::None,
                        freq: 0,
                    },
                ));

                ChannelPointer::Analog(idx as u16)
            })
            .collect();

        let scan = Scan::new(name.clone());
        let scan_list = ScanList::new(&channels);

        let scan_idx = self.scan_list.insert(scan, scan_list);

        let zone = Zone::new(name, &channels);
        let zone_list = ZoneList {
            channels,
        };

        let zone_idx = self.zones.insert(zone, zone_list);

        self.set_zone_scanlist(zone_idx, scan_idx).unwrap();

        zone_idx
    }

    pub fn mutate_cp(&mut self) {
        self.clear_codeplug();

        let mut roam_862_chans: Vec<DigiChannelPointer> = Vec::new();

        let le_zone = self.add_dmr_repeater(
            "LE",
            "Leeds",
            430_662_500,
            439_662_500,
            2,
            Some(&mut roam_862_chans),
        );
        self.add_dmr_repeater(
            "TD",
            "Wakefield",
            430_162_500,
            439_162_500,
            1,
            Some(&mut roam_862_chans),
        );
        self.add_dmr_repeater(
            "RV",
            "Ribble Val",
            430_625_000,
            439_625_000,
            2,
            Some(&mut roam_862_chans),
        );
        self.add_dmr_repeater("MP", "Heysham", 430_750_000, 439_750_000, 3, None);

        let roam_862 = Roam::new("M62 Corridor".to_string());
        let roam_862_list = RoamList::new(&roam_862_chans);

        let roam_862_idx = self.roam_list.insert(roam_862, roam_862_list);

        let simplex_zone = self.add_analog_zone(
            "70cm Simplex".to_string(),
            PowerLevel::High,
            &mut (0..8).map(|i| {
                let freq = 443_400_000 + (i * 25_000);
                (
                    format!("U{} ({:.03})", 272 + (i * 2), freq as f64 / 1000000.0),
                    freq,
                    freq,
                )
            }),
        );

        let mut home_scan_channels = self.zones.channels.data[simplex_zone].channels.clone();
        home_scan_channels.extend_from_slice(&self.zones.channels.data[le_zone].channels);

        let home_scan = Scan::new("Home".to_string());
        let home_scan_list = ScanList::new(&home_scan_channels);
        let home_scan_idx = self.scan_list.insert(home_scan, home_scan_list);

        self.set_zone_scanlist(le_zone, home_scan_idx).unwrap();

        for chan in roam_862_chans.iter() {
            if let DigiChannelPointer::Digital(di) = chan {
                    let mut dc = &mut self.digi_chans.data[*di as usize];
                    dc.ip_multi_site_connect = true;
                    dc.slrl_pointer = SLRLPointer::RoamList(roam_862_idx as u8);
                    dc.auto_start_roam = true;
            }
        }
    }
}
