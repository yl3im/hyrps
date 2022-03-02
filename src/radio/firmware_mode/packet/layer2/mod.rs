use std::{io::{Cursor, Read, Write}, convert::TryFrom};

use anyhow::{Result, bail};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use num_enum::TryFromPrimitive;

use crate::radio::common::L2Payload;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum FwMemory {
    CPU = 0x00,
    Codeplug = 0x03,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FwMemAccessStatus {
    Success,
    Fail,
}

impl From<u8> for FwMemAccessStatus {
    fn from(v: u8) -> Self {
        match v {
            0x5e => Self::Success,
            _ => Self::Fail,
        }
    }
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u16)]
pub enum FwMemAccess {
    EnableAccess = 0x10fb,
    DisableAcess = 0x0000,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FwPacketL2 {
    AccessMemoryRequest {
        access: FwMemAccess,
    },
    AccessMemoryResponse {
        status: FwMemAccessStatus,
    },
    ReadMemoryRequest {
        addr: u32,
        len: u16,
        mem: FwMemory,
    },
    ReadMemoryResponse {
        addr: u32,
        status: u8,
        mem: FwMemory,
        payload: Vec<u8>,
    },
    WriteMemoryRequest {
        mem: FwMemory,
        addr: u32,
        payload: Vec<u8>,
    },
    WriteMemoryResponse {
        status: u8,
        mem: FwMemory,
        addr: u32,
        len: u16,
    },
}

impl FwPacketL2 {
    fn unpack_access_mem(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        let access = FwMemAccess::try_from(cursor.read_u16::<LittleEndian>()?)?;

        Ok(Self::AccessMemoryRequest { access })
    }

    fn pack_access_mem(access: FwMemAccess) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        cursor.write_u16::<LittleEndian>(access as u16)?;

        Ok(ret)
    }

    fn unpack_fw_access_mem_res(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        let status = FwMemAccessStatus::from(cursor.read_u8()?);

        Ok(Self::AccessMemoryResponse { status })
    }

    fn unpack_fw_read_mem_req(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        let mem = FwMemory::try_from(cursor.read_u8()?)?;
        let addr = cursor.read_u32::<LittleEndian>()?;
        let len = cursor.read_u16::<LittleEndian>()?;

        Ok(Self::ReadMemoryRequest { addr, len, mem })
    }

    fn pack_fw_read_mem_req(addr: u32, len: u16, mem: FwMemory) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        cursor.write_u8(mem as u8)?;
        cursor.write_u32::<LittleEndian>(addr)?;
        cursor.write_u16::<LittleEndian>(len)?;

        Ok(ret)
    }

    fn unpack_fw_read_mem_res(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        let status = cursor.read_u8()?;
        let mem = FwMemory::try_from(cursor.read_u8()?)?;
        let addr = cursor.read_u32::<LittleEndian>()?;
        let payload_len = cursor.read_u16::<LittleEndian>()?;

        let mut payload = vec![0u8; payload_len as usize];

        cursor.read_exact(&mut payload)?;

        Ok(Self::ReadMemoryResponse { addr, status, mem, payload })
    }

    fn pack_fw_read_mem_res(status: u8, mem: FwMemory, addr: u32, payload: &[u8]) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        cursor.write_u8(status)?;
        cursor.write_u8(mem as u8)?;
        cursor.write_u32::<LittleEndian>(addr)?;
        cursor.write_u16::<LittleEndian>(payload.len() as u16)?;
        cursor.write_all(payload)?;

        Ok(ret)
    }

    fn unpack_fw_write_mem_req(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        let mem = FwMemory::try_from(cursor.read_u8()?)?;
        let addr = cursor.read_u32::<LittleEndian>()?;
        let payload_len = cursor.read_u16::<LittleEndian>()?;

        let mut payload = vec![0u8; payload_len as usize];

        cursor.read_exact(&mut payload)?;

        Ok(Self::WriteMemoryRequest { addr, mem, payload })
    }

    fn pack_fw_write_mem_req(mem: FwMemory, addr: u32, payload: &[u8]) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        cursor.write_u8(mem as u8)?;
        cursor.write_u32::<LittleEndian>(addr)?;
        cursor.write_u16::<LittleEndian>(payload.len() as u16)?;
        cursor.write_all(payload)?;

        Ok(ret)
    }

    fn unpack_fw_write_mem_res(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        let status = cursor.read_u8()?;
        let mem = FwMemory::try_from(cursor.read_u8()?)?;
        let addr = cursor.read_u32::<LittleEndian>()?;
        let len = cursor.read_u16::<LittleEndian>()?;

        Ok(Self::WriteMemoryResponse { status, addr, len, mem })
    }

    fn pack_fw_write_mem_res(status: u8, mem: FwMemory, addr: u32, len: u16) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        let mut cursor = Cursor::new(&mut ret);

        cursor.write_u8(status)?;
        cursor.write_u8(mem as u8)?;
        cursor.write_u32::<LittleEndian>(addr)?;
        cursor.write_u16::<LittleEndian>(len)?;

        Ok(ret)
    }
}

impl L2Payload for FwPacketL2 {
    fn deser_payload(id: u16, data: &[u8]) -> Result<Self> {
        match id {
            0x01c2 => Self::unpack_fw_read_mem_req(data),
            0x81c2 => Self::unpack_fw_read_mem_res(data),
            0x01c3 => Self::unpack_fw_write_mem_req(data),
            0x81c3 => Self::unpack_fw_write_mem_res(data),
            0x0218 => Self::unpack_access_mem(data),
            0x8218 => Self::unpack_fw_access_mem_res(data),
            _ => bail!("Unknown Firmware payload ID {}", id)
        }
    }

    fn get_id(&self) -> u16 {
        match self {
            Self::ReadMemoryRequest { .. } => 0x01c2,
            Self::ReadMemoryResponse { .. } => 0x81c2,
            Self::WriteMemoryRequest { .. } => 0x01c3,
            Self::WriteMemoryResponse { .. } => 0x81c3,
            Self::AccessMemoryRequest { .. } => 0x0218,
            Self::AccessMemoryResponse { .. } => 0x8218,
        }
    }

    fn ser_payload(&self) -> Result<Vec<u8>> {
        match self {
            Self::ReadMemoryRequest { addr, len, mem } => Self::pack_fw_read_mem_req(*addr, *len, *mem),
            Self::ReadMemoryResponse { addr, status, mem, payload } => Self::pack_fw_read_mem_res(*status, *mem, *addr, payload),
            Self::WriteMemoryRequest { mem, addr, payload } => Self::pack_fw_write_mem_req(*mem, *addr, payload),
            Self::WriteMemoryResponse { status, mem, addr, len } => Self::pack_fw_write_mem_res(*status, *mem, *addr, *len),
            Self::AccessMemoryRequest { access } => Self::pack_access_mem(*access),
            _ => bail!("Packing {self:?} is unsupported"),
        }
    }
}
