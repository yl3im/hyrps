use std::io::{Cursor, Write, Read};

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(test)]
use proptest_derive::Arbitrary;


pub trait L2Payload
where Self: Sized{
    fn ser_payload(&self) -> Result<Vec<u8>>;
    fn get_id(&self) -> u16;
    fn deser_payload(id: u16, data: &[u8]) -> Result<Self>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct L2 <T: L2Payload> {
    pub payload: T,
}

impl<T: L2Payload> L2<T> {
    pub fn new(payload: T) -> Self {
        L2 { payload }
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        let payload = self.payload.ser_payload()?;
        let id = self.payload.get_id();

        cursor.write_u8(2)?;

        cursor.write_u16::<LittleEndian>(id)?;

        cursor.write_u16::<LittleEndian>(payload.len() as u16)?;
        cursor.write_all(&payload)?;

        let crc: u32 = ret[1..].iter().map(|&x| x as u32).sum();

        ret.push((((!crc).overflowing_add(0x33).0) & 0xff) as u8);
        ret.push(3);

        Ok(ret)
    }

    pub fn unpack(data: Vec<u8>) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        assert_eq!(cursor.read_u8()?, 0x2);

        let kind = cursor.read_u16::<LittleEndian>()?;

        let mut payload = vec![0; cursor.read_u16::<LittleEndian>()? as usize];

        cursor.read_exact(&mut payload)?;

        let _crc = cursor.read_u8()?;

        assert_eq!(cursor.read_u8()?, 3);

        Ok(L2 {
            payload: T::deser_payload(kind, &payload)?,
        })
    }
}
