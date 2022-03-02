use super::{CPSPacketL2, StringReqType};
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::TryFrom,
    io::{Cursor, Read, Write},
};

pub fn pack_get_string_request(what: StringReqType) -> Result<Vec<u8>> {
    let ret = vec![what as u8];

    Ok(ret)
}

pub fn unpack_get_string_request(data: &[u8]) -> Result<CPSPacketL2> {
    assert_eq!(data.len(), 1);

    let what = StringReqType::try_from(data[0])?;

    Ok(CPSPacketL2::GetStringRequest { what })
}

pub fn pack_get_string_response(what: StringReqType, s: &str) -> Result<Vec<u8>> {
    let mut ret = Vec::new();
    let mut cursor = Cursor::new(&mut ret);

    cursor.write_all(&[0u8; 4])?;

    cursor.write_u8(what as u8)?;

    cursor.write_all(&[0u8; 3])?;

    s.encode_utf16()
        .try_for_each(|x| cursor.write_u16::<LittleEndian>(x))?;

    Ok(ret)
}

pub fn unpack_get_string_response(data: &[u8]) -> Result<CPSPacketL2> {
    let mut cursor = Cursor::new(&data);
    let mut buf = [0u8; 3];
    let mut char_buf = vec![0u16; (data.len() - 8) >> 1];

    let _ = cursor.read_u8();

    cursor.read_exact(&mut buf)?;

    assert_eq!(buf, [0u8; 3]);

    let what = StringReqType::try_from(cursor.read_u8()?)?;

    cursor.read_exact(&mut buf)?;

    assert_eq!(buf, [0u8; 3]);

    cursor.read_u16_into::<LittleEndian>(&mut char_buf)?;

    let str = String::from_utf16(&char_buf)?;

    Ok(CPSPacketL2::GetStringResponse { what, str })
}
