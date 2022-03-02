use super::CPSPacketL2;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

static HEADER: [u8; 6] = [0, 0, 0, 1, 0, 0];

pub fn pack_read_codeplug_request(addr: u32, len: u16) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_all(&HEADER)?;
    cursor.write_u32::<LittleEndian>(addr)?;
    cursor.write_u16::<LittleEndian>(len)?;

    Ok(ret)
}

pub fn unpack_read_codeplug_request(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);

    let mut hdr = vec![0; HEADER.len()];

    cursor.read_exact(&mut hdr)?;

    assert_eq!(hdr, HEADER);

    let addr = cursor.read_u32::<LittleEndian>()?;
    let len = cursor.read_u16::<LittleEndian>()?;

    Ok(CPSPacketL2::ReadCodeplugRequest { addr, len })
}

pub fn pack_read_codeplug_response(addr: u32, payload: &[u8]) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_u8(0)?;
    cursor.write_all(&HEADER)?;
    cursor.write_u32::<LittleEndian>(addr)?;
    cursor.write_u16::<LittleEndian>(payload.len() as u16)?;
    cursor.write_all(payload)?;

    Ok(ret)
}

pub fn unpack_read_codeplug_response(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);

    assert_eq!(cursor.read_u8()?, 0);

    let mut hdr = vec![0; HEADER.len()];

    cursor.read_exact(&mut hdr)?;

    assert_eq!(hdr, HEADER);

    let addr = cursor.read_u32::<LittleEndian>()?;
    let len = cursor.read_u16::<LittleEndian>()?;

    let mut payload = vec![0; len as usize];

    cursor.read_exact(&mut payload)?;

    Ok(CPSPacketL2::ReadCodeplugResponse { addr, payload })
}

pub fn pack_write_codeplug_request(addr: u32, payload: &[u8]) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_all(&HEADER)?;
    cursor.write_u32::<LittleEndian>(addr)?;
    cursor.write_u16::<LittleEndian>(payload.len() as u16)?;
    cursor.write_all(payload)?;

    Ok(ret)
}

pub fn unpack_write_codeplug_request(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);

    let mut hdr = vec![0; HEADER.len()];

    cursor.read_exact(&mut hdr)?;

    assert_eq!(hdr, HEADER);

    let addr = cursor.read_u32::<LittleEndian>()?;
    let len = cursor.read_u16::<LittleEndian>()?;
    let mut payload = vec![0; len as usize];
    cursor.read_exact(&mut payload)?;

    Ok(CPSPacketL2::WriteCodeplugRequest { addr, payload })
}

pub fn pack_write_codeplug_response(addr: u32, len: u16) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_u8(0)?;
    cursor.write_all(&HEADER)?;
    cursor.write_u32::<LittleEndian>(addr)?;
    cursor.write_u16::<LittleEndian>(len)?;

    Ok(ret)
}

pub fn unpack_write_codeplug_response(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);

    assert_eq!(cursor.read_u8()?, 0);

    let mut hdr = vec![0; HEADER.len()];

    cursor.read_exact(&mut hdr)?;

    assert_eq!(hdr, HEADER);

    let addr = cursor.read_u32::<LittleEndian>()?;
    let len = cursor.read_u16::<LittleEndian>()?;

    Ok(CPSPacketL2::WriteCodeplugResponse { addr, len })
}
