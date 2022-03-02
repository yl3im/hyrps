use self::packet::{layer1::FwPacketL1, layer2::{FwPacketL2, FwMemory, FwMemAccess, FwMemAccessStatus}};
use std::time::Duration;

use super::prog_mode::ProgMode;
use super::Radio;
use anyhow::{bail, Context, Result};

mod packet;

#[derive(Clone, Copy)]
pub struct FirmwareMode {
}

impl FirmwareMode {
    pub fn new() -> FirmwareMode {
        FirmwareMode {}
    }
}

fn xfer_l1<T: ProgMode>(radio: &Radio<T>, pkt: &FwPacketL1) -> Result<FwPacketL1> {
    let timeout = Duration::from_secs(10);
    let xmit_buf = pkt.pack().context("Failed to pack xmit packet")?;
    let mut buf = vec![0u8; 1024];

    if radio.verbose {
        println!();
        println!("--------------------------------------");
        println!("REQ: {:?}", pkt);
    }

    radio
        .handle
        .write_bulk(radio.ep_out, &xmit_buf, timeout)
        .context("Failed to write to device")?;

    let n = radio
        .handle
        .read_bulk(radio.ep_in, &mut buf, timeout)
        .context("Failed to read from device")?;

    buf.resize(n, 0);

    let ret = FwPacketL1::unpack(&buf).context("Failed to unpack device response")?;

    if radio.verbose {
        println!("RES: {:?}", ret);
    }

    Ok(ret)
}

fn xfer<T: ProgMode>(radio: &Radio<T>, pkt: FwPacketL2) -> Result<FwPacketL2> {
    let xmit_pkt = FwPacketL1::new(pkt);

    let response = xfer_l1(radio, &xmit_pkt)?;

    Ok(response.payload.payload)
}

impl ProgMode for FirmwareMode {
    fn open<T: ProgMode>(&mut self, radio: &Radio<T>) -> Result<()> {
        let response = xfer(radio, FwPacketL2::AccessMemoryRequest { access: FwMemAccess::EnableAccess})?;

        match response {
            FwPacketL2::AccessMemoryResponse { status } => {
                assert_eq!(status, FwMemAccessStatus::Success);
            }
            _ => bail!("Unexpected response when enabling firmware memory {:?}", response)
        }

        Ok(())
    }

    fn get_vid_pid_eps() -> Vec<(u16, u16, u8)> {
        Vec::from([(0x8765, 0x1234, 0x01)])
    }

    fn get_chunk_sz() -> usize {
        128
    }

    fn read<T: ProgMode>(self, radio: &Radio<T>, x: &mut [u8]) -> Result<()> {
        let request = FwPacketL2::ReadMemoryRequest {
            addr: radio.pos as u32,
            len: x.len() as u16,
            mem: FwMemory::Codeplug,
        };

        let response = xfer(radio, request)?;

        match response {
            FwPacketL2::ReadMemoryResponse { addr, payload, status, mem } => {
                assert_eq!(addr as usize, radio.pos);
                assert_eq!(payload.len(), x.len());
                assert_eq!(mem, FwMemory::Codeplug);
                assert_eq!(status, 0);

                x.copy_from_slice(&payload);
            }
            _ =>  bail!("Unexpected response to read codeplug request: {:?}", response),
        }

        Ok(())
    }

    fn write<T: ProgMode>(self, radio: &Radio<T>, x: &[u8]) -> Result<()> {
        let request = FwPacketL2::WriteMemoryRequest  {
            addr: radio.pos as u32,
            payload: x.to_vec(),
            mem: FwMemory::Codeplug,
        };

        let response = xfer(radio, request)?;

        match response {
            FwPacketL2::WriteMemoryResponse { status, mem, addr, len } => {
                assert_eq!(addr as usize, radio.pos);
                assert_eq!(len, x.len() as u16);
                assert_eq!(status, 0);
                assert_eq!(mem, FwMemory::Codeplug);
            }
            _ => bail!("Unexpected response to write codeplug request: {:?}", response),
        }

        Ok(())
    }

    fn drop<T: ProgMode>(self, radio: &Radio<T>) {
        let response = xfer(radio, FwPacketL2::AccessMemoryRequest { access: FwMemAccess::DisableAcess}).unwrap();

        match response {
            FwPacketL2::AccessMemoryResponse { status } => {
                assert_eq!(status, FwMemAccessStatus::Success);
            }
            _ => panic!("Unexpected response when disabling firmware memory {:?}", response)
        }
    }
}
