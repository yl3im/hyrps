use std::io::Write;

use anyhow::{Result, bail};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::radio::common::L2;

use super::layer2::FwPacketL2;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FwPacketL1 {
    pub payload: L2<FwPacketL2>,
}

static HDR_LEN: u16 = 4;

impl FwPacketL1 {
    pub fn new(payload: FwPacketL2) -> Self {
        Self {
            payload: L2::new(payload),
        }
    }

    fn pack_hdr(&self, crc: u16, v: &mut Vec<u8>) -> Result<()> {
        let mut cursor = std::io::Cursor::new(v);
        let payload_data = self.payload.pack()?;

        cursor.write_u16::<LittleEndian>(payload_data.len() as u16 + HDR_LEN)?;
        cursor.write_u16::<LittleEndian>(crc)?;
        cursor.write_all(&payload_data)?;

        Ok(())
    }

    pub fn unpack(data: &[u8]) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        let total_len = cursor.read_u16::<LittleEndian>()?;
        let _crc = cursor.read_u16::<LittleEndian>()?;

        let payload_len = total_len - HDR_LEN;
        let payload = match payload_len {
            0 => bail!("Empty payload found in firmware packet"),
            _ => L2::unpack(data[HDR_LEN as usize..].to_vec())?,
        };

        Ok(FwPacketL1 {
            payload,
        })
    }

    fn crc(payload: &Vec<u8>) -> u16 {
        let mut cursor = std::io::Cursor::new(payload);

        let xform = |x: u32| (!x) & 0xffff;

        let mut n: u32 = xform(payload.len() as u32 + HDR_LEN as u32);

         for _ in 0..payload.len() >> 1 {
             n +=  xform(cursor.read_u16::<LittleEndian>().unwrap() as u32);
         }

        if payload.len() % 2 == 1 {
            n += xform((*payload.last().unwrap() as u32) << 8);
        }

        (n & 0xffff) as u16
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let mut v = Vec::new();

        self.pack_hdr(0, &mut v)?;

        let crc = Self::crc(&self.payload.pack()?);

        v.clear();

        self.pack_hdr(crc, &mut v)?;

        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::FwPacketL1;

    #[test]
    fn known_crc() {
        let tests = vec![
            (vec![], 65531_u16),
            (vec![0xde, 0xad, 0xbe, 0xef], 25177_u16),
            (vec![0xde, 0xad, 0xbe, 0xef, 0xad], 46423_u16),
            (vec![0x02, 0x18, 0x02, 0x02, 0x00, 0xfb, 0x10, 0x0b, 0x03], 56537_u16),
        ];

        for (test, res) in tests.iter() {
            assert_eq!(FwPacketL1::crc(test), *res);
        }

    }
}
