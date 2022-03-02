use super::{OpenMode, CPSPacketL2};
use anyhow::{Result, bail};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::TryFrom,
    io::{Cursor, Read, Write},
};

static FOOTER: [u8; 16] = [0; 16];

pub fn pack_enter_prog_mode_request(what: OpenMode) -> Result<Vec<u8>> {
    let mut ret = Vec::new();

    ret.push(what as u8);
    ret.extend(FOOTER);

    Ok(ret)
}

pub fn unpack_enter_prog_mode_request(data: &[u8]) -> Result<CPSPacketL2> {
    if data.len() != FOOTER.len() + 1 {
        bail!("Invalid payload length");
    }

    let mode = OpenMode::try_from(data[0])?;

    assert_eq!(data[1..], FOOTER);

    Ok(CPSPacketL2::EnterProogModeRequest { mode })
}

pub fn pack_enter_prog_mode_response(
    status: u8,
    mode: OpenMode,
    head_data: &[u8],
) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_u8(status)?;
    cursor.write_u8(mode as u8)?;
    cursor.write_u16::<LittleEndian>(head_data.len() as u16)?;
    cursor.write_all(head_data)?;

    Ok(ret)
}

pub fn unpack_enter_prog_mode_response(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);

    let status = cursor.read_u8()?;

    let mode = OpenMode::try_from(cursor.read_u8()?)?;

    let mut head_data = vec![0; cursor.read_u16::<LittleEndian>()? as usize];

    cursor.read_exact(&mut head_data)?;

    Ok(CPSPacketL2::EnterProogModeResponse {
        status,
        mode,
        head_data,
    })
}

pub fn pack_leave_prog_mode_request() -> Result<Vec<u8>> {
    let ret = vec![0];

    Ok(ret)
}

pub fn unpack_leave_prog_mode_request(data: &[u8]) -> Result<CPSPacketL2> {
    assert_eq!(data.len(), 1);

    Ok(CPSPacketL2::LeaveProgModeRequest)
}

pub fn pack_leave_prog_mode_response() -> Result<Vec<u8>> {
    let ret = vec![0];

    Ok(ret)
}

pub fn unpack_leave_prog_mode_response(data: &[u8]) -> Result<CPSPacketL2> {
    assert_eq!(data.len(), 1);

    Ok(CPSPacketL2::LeaveProgModeResponse)
}
