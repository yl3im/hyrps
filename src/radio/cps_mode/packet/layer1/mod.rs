use std::{convert::TryFrom, io::Write};

use crate::radio::common::L2;

use super::layer2::CPSPacketL2;
use anyhow::Result;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;

#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Command {
    Connect = 0x00,
    Req = 0x01,
    Res = 0x04,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Flags {
    Default = 0x00,
    Connnect = 0xfe,
    ResConnect = 0xfd,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Entity {
    Host = 0x20,
    Radio = 0x10,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct PacketL1 {
    pub command: Command,
    pub flags: Flags,
    pub src: Entity,
    pub dst: Entity,
    pub seq: u16,
    pub payload: Option<L2<CPSPacketL2>>,
}

fn error_xform(e: impl std::string::ToString) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

static HDR_LEN: u16 = 12;

impl PacketL1 {
    pub fn new(command: Command, flags: Flags, payload: Option<L2<CPSPacketL2>>) -> Self {
        PacketL1 {
            command,
            flags,
            src: Entity::Host,
            dst: Entity::Radio,
            seq: 0,
            payload,
        }
    }

    fn pack_hdr(&self, crc: u16, v: &mut Vec<u8>) -> Result<()> {
        let mut cursor = std::io::Cursor::new(v);
        let payload_data = self.payload.as_ref().map(|x| x.pack()).transpose()?;

        cursor.write_u8(0x7e)?;
        cursor.write_u8(self.command as u8)?;
        cursor.write_u8(0x0)?;
        cursor.write_u8(self.flags as u8)?;
        cursor.write_u8(self.src as u8)?;
        cursor.write_u8(self.dst as u8)?;
        cursor.write_u16::<BigEndian>(self.seq)?;
        cursor.write_u16::<BigEndian>(
            HDR_LEN + payload_data.as_ref().map_or(0, |x| x.len()) as u16,
        )?;
        cursor.write_u16::<LittleEndian>(crc)?;
        payload_data.map(|x| cursor.write_all(&x)).transpose()?;

        Ok(())
    }

    pub fn unpack(data: &[u8]) -> Result<PacketL1> {
        let mut cursor = std::io::Cursor::new(data);

        assert_eq!(cursor.read_u8()?, 0x7e);

        let command = Command::try_from(cursor.read_u8()?).map_err(error_xform)?;
        assert_eq!(cursor.read_u8()?, 0x0);
        let flags = Flags::try_from(cursor.read_u8()?).map_err(error_xform)?;
        let src = Entity::try_from(cursor.read_u8()?).map_err(error_xform)?;
        let dst = Entity::try_from(cursor.read_u8()?).map_err(error_xform)?;
        let seq = cursor.read_u16::<BigEndian>()?;
        let total_len = cursor.read_u16::<BigEndian>()?;
        let _crc = cursor.read_u16::<LittleEndian>()?;

        let payload_len = total_len - HDR_LEN;
        let payload = match payload_len {
            0 => None,
            _ => Some(L2::unpack(data[HDR_LEN as usize..].to_vec())?),
        };

        Ok(PacketL1 {
            command,
            flags,
            src,
            dst,
            seq,
            payload,
        })
    }

    fn crc(data: &Vec<u8>) -> u16 {
        let mut cursor = std::io::Cursor::new(data);

        let mut n: u32 = (0..data.len() >> 1)
            .map(|_| cursor.read_u16::<LittleEndian>().unwrap() as u32)
            .sum();

        if data.len() % 2 == 1 {
            n += *data.last().unwrap() as u32;
        }

        while n > 0xffff {
            n = (n >> 16) + (n & 0xffff);
        }

        (n as u16) ^ 0xffff
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let mut v = Vec::new();

        self.pack_hdr(0, &mut v)?;

        let crc = Self::crc(&v);

        v.clear();

        self.pack_hdr(crc, &mut v)?;

        Ok(v)
    }
}

#[cfg(test)]
proptest! {
    #[test]
    fn packet_l1_serial_deserialise(pkt in any::<PacketL1>()) {
        let data = pkt.pack().unwrap();

        let x = PacketL1::unpack(&data).unwrap();

        prop_assert_eq!(x, pkt);
    }

}

#[test]
fn packet_l1_known_crc() {
    let mut buf = Vec::new();
    let data = vec![
        (
            PacketL1::new(Command::Connect, Flags::Connnect, None),
            0xe560u16,
        ),
        (
            PacketL1::new(
                Command::Req,
                Flags::Default,
                Some(L2::new(CPSPacketL2::ReadCodeplugRequest {
                    addr: 0,
                    len: 0x100,
                })),
            ),
            0xa058u16,
        ),
    ];

    for (pkt, crc) in data.iter() {
        pkt.pack_hdr(0, &mut buf).unwrap();

        let tst_crc = PacketL1::crc(&buf);

        assert_eq!(tst_crc, *crc);
    }
}

#[test]
fn packet_l1_payload() {
    let payload = L2::new(CPSPacketL2::LeaveProgModeRequest);
    let pkt = PacketL1::new(Command::Connect, Flags::Connnect, Some(payload.clone()));

    assert_eq!(
        pkt.pack().unwrap()[HDR_LEN as usize..],
        payload.pack().unwrap()
    );
}
