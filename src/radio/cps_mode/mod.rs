pub use self::packet::layer2::OpenMode;
use std::time::Duration;

use self::packet::{
    layer1::{Command, Entity, Flags, PacketL1},
    layer2::CPSPacketL2,
};
use super::{prog_mode::ProgMode, common::L2};
use super::Radio;
use anyhow::{bail, Context, Result};

mod packet;

#[derive(Clone, Copy)]
pub struct CPSMode {
    mode: OpenMode,
    entered_prog_mode: bool,
}

impl CPSMode {
    pub fn new(mode: OpenMode) -> CPSMode {
        CPSMode {
            mode,
            entered_prog_mode: false,
        }
    }
}

fn xfer_l1<T: ProgMode>(radio: &Radio<T>, pkt: &PacketL1) -> Result<PacketL1> {
    let timeout = Duration::new(4, 0);
    let xmit_buf = pkt.pack().context("Failed to pack xmit packet")?;
    let mut buf = [0u8; 1024];

    if radio.verbose {
        println!();
        println!("--------------------------------------");
        println!("REQ: {:?}", pkt);
    }

    radio
        .handle
        .write_bulk(radio.ep_out, &xmit_buf, timeout)
        .context("Failed to write to device")?;

    radio
        .handle
        .read_bulk(radio.ep_in, &mut buf, timeout)
        .context("Failed to read from device")?;

    let ret = PacketL1::unpack(&buf).context("Failed to unpack device response")?;

    if radio.verbose {
        println!("RES: {:?}", ret);
    }

    Ok(ret)
}

fn xfer<T: ProgMode>(radio: &Radio<T>, pkt: CPSPacketL2) -> Result<Option<CPSPacketL2>> {
    let xmit_pkt = PacketL1::new(Command::Req, Flags::Default, Some(L2::new(pkt)));

    let response = xfer_l1(radio, &xmit_pkt)?;

    match response {
        PacketL1 {
            command: Command::Res,
            flags: Flags::Default,
            src: Entity::Radio,
            dst: Entity::Host,
            ..
        } => (),
        _ => bail!("Unexpected response in layer 1: {:?}", response),
    };

    Ok(response.payload.map(|x| x.payload))
}

impl ProgMode for CPSMode {
    fn open<T: ProgMode>(&mut self, radio: &Radio<T>) -> Result<()> {
        let pkt_connect = PacketL1::new(
            packet::layer1::Command::Connect,
            packet::layer1::Flags::Connnect,
            None,
        );

        let result = xfer_l1(radio, &pkt_connect).context("Handshake failed")?;

        match result {
            PacketL1 {
                command: Command::Res,
                flags: Flags::ResConnect,
                ..
            } => (),
            _ => bail!("Unexpected handshake response: {:?}", result),
        };

        let ep_result = xfer(radio, CPSPacketL2::EnterProogModeRequest { mode: self.mode })
            .context("Failed to enter programming mode")?;

        match ep_result {
            Some(CPSPacketL2::EnterProogModeResponse {
                status: 0, mode, ..
            }) => assert_eq!(mode, self.mode),
            _ => bail!(
                "Unexpected enter programming mode response: {:?}",
                ep_result
            ),
        };

        self.entered_prog_mode = true;

        Ok(())
    }

    fn get_vid_pid_eps() -> Vec<(u16, u16, u8)> {
        Vec::from([(0x0a11, 0x04), (0x0a12, 0x02), (0x0a21, 0x04)])
            .iter()
            .map(|x| (0x238b, x.0, x.1))
            .collect()
    }

    fn get_chunk_sz() -> usize {
        0x100
    }

    fn read<T: ProgMode>(self, radio: &Radio<T>, x: &mut [u8]) -> Result<()> {
        let request = CPSPacketL2::ReadCodeplugRequest {
            addr: radio.pos as u32,
            len: x.len() as u16,
        };

        let response = xfer(radio, request).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                format!("xfer failed: {:#}", e),
            )
        })?;

        match response {
            Some(CPSPacketL2::ReadCodeplugResponse { addr, payload }) => {
                assert_eq!(addr as usize, radio.pos);
                assert_eq!(payload.len(), x.len());

                x.copy_from_slice(&payload);

                Ok(())
            }
            _ => bail!("Unexpected response to read codeplug request: {:?}", response),
        }
    }

    fn write<T: ProgMode>(self, radio: &Radio<T>, x: &[u8]) -> Result<()>{
        let request = CPSPacketL2::WriteCodeplugRequest {
            addr: radio.pos as u32,
            payload: x.to_vec(),
        };

        let response = xfer(radio, request).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                format!("xfer failed: {:#}", e),
            )
        })?;

        match response {
            Some(CPSPacketL2::WriteCodeplugResponse { addr, len }) => {
                assert_eq!(addr as usize, radio.pos);
                assert_eq!(len, x.len() as u16);

                Ok(())
            },
            _ => bail!("Unexpected response to write codeplug request: {:?}", response),
        }
    }

    fn drop<T: ProgMode>(self, radio: &Radio<T>) {
        if !self.entered_prog_mode {
            return;
        }

        let result = xfer(radio, CPSPacketL2::LeaveProgModeRequest).unwrap();

        match result {
            Some(CPSPacketL2::LeaveProgModeResponse) => (),
            _ => panic!("Unexpected leave programming mode response: {:?}", result),
        }
    }
}
