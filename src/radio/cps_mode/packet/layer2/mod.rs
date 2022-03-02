use crate::radio::common::L2Payload;

use self::string::{unpack_get_string_request, unpack_get_string_response};
use anyhow::{Result, bail};

use num_enum::TryFromPrimitive;


use self::{
    codeplug_request::*,
    open_mode::*,
    string::{pack_get_string_request, pack_get_string_response},
};

#[cfg(test)]
use proptest_derive::Arbitrary;

mod codeplug_request;
mod open_mode;
mod string;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum StringReqType {
    RadioID = 0x00,
    Unknown1 = 0x12,
    Unknown2 = 0x09,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum OpenMode {
    Read = 0x00,
    Write = 0x02,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CPSPacketL2 {
    ReadCodeplugRequest {
        addr: u32,
        len: u16,
    },
    ReadCodeplugResponse {
        addr: u32,
        payload: Vec<u8>,
    },
    WriteCodeplugRequest {
        addr: u32,
        payload: Vec<u8>,
    },
    WriteCodeplugResponse {
        addr: u32,
        len: u16,
    },
    GetStringRequest {
        what: StringReqType,
    },
    GetStringResponse {
        what: StringReqType,
        str: String,
    },
    EnterProogModeRequest {
        mode: OpenMode,
    },
    EnterProogModeResponse {
        status: u8,
        mode: OpenMode,
        head_data: Vec<u8>,
    },
    LeaveProgModeRequest,
    LeaveProgModeResponse,
}

impl L2Payload for CPSPacketL2 {
    fn ser_payload(&self) -> Result<Vec<u8>> {
        match self {
            CPSPacketL2::ReadCodeplugRequest { addr, len } => pack_read_codeplug_request(*addr, *len),
            CPSPacketL2::ReadCodeplugResponse { addr, payload } => {
                pack_read_codeplug_response(*addr, payload)
            }
            CPSPacketL2::WriteCodeplugRequest { addr, payload } => {
                pack_write_codeplug_request(*addr, payload)
            }
            CPSPacketL2::WriteCodeplugResponse { addr, len } => {
                pack_write_codeplug_response(*addr, *len)
            }
            CPSPacketL2::EnterProogModeRequest { mode } => pack_enter_prog_mode_request(*mode),
            CPSPacketL2::EnterProogModeResponse {
                status,
                mode,
                head_data,
            } => pack_enter_prog_mode_response(*status, *mode, head_data),
            CPSPacketL2::LeaveProgModeRequest => pack_leave_prog_mode_request(),
            CPSPacketL2::LeaveProgModeResponse => pack_leave_prog_mode_response(),
            CPSPacketL2::GetStringRequest { what } => pack_get_string_request(*what),
            CPSPacketL2::GetStringResponse { what, str } => pack_get_string_response(*what, str),
        }
    }

    fn get_id(&self) -> u16 {
        match self {
            CPSPacketL2::EnterProogModeRequest { .. } => 0x01c5,
            CPSPacketL2::EnterProogModeResponse { .. } => 0x81c5,
            CPSPacketL2::LeaveProgModeRequest { .. } => 0x01c6,
            CPSPacketL2::LeaveProgModeResponse { .. } => 0x81c6,
            CPSPacketL2::ReadCodeplugRequest { .. } => 0x01c7,
            CPSPacketL2::ReadCodeplugResponse { .. } => 0x81c7,
            CPSPacketL2::WriteCodeplugRequest { .. } => 0x01c8,
            CPSPacketL2::WriteCodeplugResponse { .. } => 0x81c8,
            CPSPacketL2::GetStringRequest { .. } => 0x0203,
            CPSPacketL2::GetStringResponse { .. } => 0x8203,
        }
    }

    fn deser_payload(id: u16, data: &[u8]) -> Result<Self> {
        match id {
            0x01c5 => unpack_enter_prog_mode_request(data),
            0x81c5 => unpack_enter_prog_mode_response(data),
            0x01c6 => unpack_leave_prog_mode_request(data),
            0x81c6 => unpack_leave_prog_mode_response(data),
            0x01c7 => unpack_read_codeplug_request(data),
            0x81c7 => unpack_read_codeplug_response(data),
            0x01c8 => unpack_write_codeplug_request(data),
            0x81c8 => unpack_write_codeplug_response(data),
            0x0203 => unpack_get_string_request(data),
            0x8203 => unpack_get_string_response(data),
            _ => bail!("Unknown layer 2 packet kind"),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
use crate::radio::common::L2Payload;

    proptest! {
        #[test]
        fn packet_cpsl2_serial_deserialise(pkt in any::<super::CPSPacketL2>()) {
            let data = pkt.ser_payload().unwrap();
            let id = pkt.get_id();

            let x = super::CPSPacketL2::deser_payload(id, &data).unwrap();

            assert_eq!(x, pkt);
        }
    }
}
