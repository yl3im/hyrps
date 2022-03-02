use self::prog_mode::ProgMode;

use anyhow::{anyhow, Context as ErrContext, Result};
use rusb::{Context, DeviceHandle, UsbContext};
use std::io::{Read, Seek, Write};

pub mod cps_mode;
pub mod firmware_mode;
mod prog_mode;
mod common;

pub struct Radio<T: ProgMode> {
    handle: DeviceHandle<Context>,
    pos: usize,
    entered_prog_mode: bool,
    verbose: bool,
    ep_out: u8,
    ep_in: u8,
    prog_mode: T,
}

pub const CPS_MEM_MAX_SZ: usize = 0x1A5B00;

impl<T: ProgMode> Radio<T> {
    pub fn new(verbose: bool, mut prog_mode: T) -> Result<Self> {
        let ctx = Context::new().context("Could not get USB context")?;

        let mut maybe_dev = None;

        for d in ctx
            .devices()
            .context("Could not enumerate USB devices")?
            .iter()
        {
            let dc = d
                .device_descriptor()
                .context("Could not get USB device descriptor")?;

            for radio in T::get_vid_pid_eps() {
                if dc.vendor_id() == radio.0 && dc.product_id() == radio.1 {
                    assert_eq!(dc.num_configurations(), 1);
                    maybe_dev = Some((d, radio.2));
                    break;
                }
            }
        }

        let dev = maybe_dev.ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Radio not found",
        ))?;

        let cfg = dev.0.config_descriptor(0)?;

        let iface = cfg
            .interfaces()
            .find(|i| {
                for id in i.descriptors() {
                    for ep in id.endpoint_descriptors() {
                        if ep.number() == dev.1 {
                            return true;
                        }
                    }
                }
                false
            })
            .ok_or_else(|| anyhow!("Could not find configuration endpoint"))?;

        let mut handle = dev.0.open().context("Could not open USB Device")?;

        handle
            .reset()
            .context("Failed to reset USB device")?;

        handle
            .set_active_configuration(cfg.number())
            .context("Failed to set USB configuration")?;

        handle
            .claim_interface(iface.number())
            .context("Could not claim USB device interface")?;

        let mut ret = Radio {
            handle,
            pos: 0,
            entered_prog_mode: false,
            verbose,
            ep_out: dev.1,
            ep_in: dev.1 | 0x80,
            prog_mode,
        };

        prog_mode.open(&ret)?;

        ret.entered_prog_mode = true;

        Ok(ret)
    }

    fn calc_bytes_to_copy(&self, buf: &[u8]) -> usize {
        std::cmp::min(T::get_chunk_sz(),
                      std::cmp::min(self.bytes_left(), buf.len()))
    }

    /// Given a buffer `buf`, return a new slice which is truncated to the
    /// maximum memory size if it overlaps it given `self.pos`.
    fn resize_buf<'a>(&self, buf: &'a [u8]) -> &'a [u8] {
        let n = self.calc_bytes_to_copy(buf);

        &buf[..n]
    }

    fn resize_buf_mut<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        let n = self.calc_bytes_to_copy(buf);

        &mut buf[..n]
    }

    fn bytes_left(&self) -> usize {
        CPS_MEM_MAX_SZ - self.pos
    }
}

fn conv_err(e: &anyhow::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

impl<T: ProgMode> Write for Radio<T> {
    fn write(&mut self, x: &[u8]) -> std::io::Result<usize> {
        let buf = self.resize_buf(x);

        self.prog_mode.write(self, buf).map_err(|e| conv_err(&e))?;

        self.pos += buf.len();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T: ProgMode> Seek for Radio<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(n) => {
                self.pos = n as usize;
            }
            std::io::SeekFrom::Current(n) => {
                if n < 0 {
                    self.pos -= n as usize;
                } else {
                    self.pos += n as usize;
                }
            }
            std::io::SeekFrom::End(n) => {
                self.pos = CPS_MEM_MAX_SZ - n as usize;
            }
        };

        Ok(self.pos as u64)
    }
}

impl<T: ProgMode> Read for Radio<T> {
    fn read(&mut self, x: &mut [u8]) -> std::io::Result<usize> {
        let buf = self.resize_buf_mut(x);

        self.prog_mode.read(self, buf).map_err(|e| conv_err(&e))?;

        self.pos += buf.len();

        Ok(buf.len())
    }
}

impl<T: ProgMode> Drop for Radio<T> {
    fn drop(&mut self) {
        self.prog_mode.drop(self)
    }
}
