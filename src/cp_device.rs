use super::Radio;
use crate::radio::cps_mode::{CPSMode, OpenMode};
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};

pub trait CPDevice: Read + Seek + Write {}

impl CPDevice for File {}

impl CPDevice for Radio<CPSMode> {}

pub fn get_source(path: &Option<std::path::PathBuf>, verbose: bool) -> Result<Box<dyn CPDevice>> {
    Ok(match path {
        Some(p) => Box::new(File::open(p).context("Failed to open input file")?),
        None => Box::new(
            Radio::new(verbose, CPSMode::new(OpenMode::Read)).context("Failed to open radio")?,
        ),
    })
}

pub fn get_sink(path: &Option<std::path::PathBuf>, verbose: bool) -> Result<Box<dyn CPDevice>> {
    Ok(match path {
        Some(p) => Box::new(
            OpenOptions::new()
                .write(true)
                .open(p)
                .context("Failed to open output file")?,
        ),
        None => Box::new(
            Radio::new(verbose, CPSMode::new(OpenMode::Read)).context("Failed to open radio")?,
        ),
    })
}
