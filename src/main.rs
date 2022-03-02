use crate::codeplug::section::Section;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codeplug::Codeplug;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};
use cp_device::{get_sink, get_source};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use radio::{
    cps_mode::{CPSMode, OpenMode},
    Radio, firmware_mode::FirmwareMode, CPS_MEM_MAX_SZ,
};
use std::{
    fs::File,
    io::{Read, Write},
};

mod codeplug;
mod cp_device;
mod custom_cp;
mod radio;

#[cfg(test)]
mod tests;

#[derive(Parser, Debug)]
/// Hytera code plug editor - Swiss army knife for interfacing with Hytera
/// radios.
struct Args {
    /// Log all commands sent to and recieved from the radio.
    #[clap(short, long)]
    verbose: bool,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Dump a codeplug memory image to a file.
    DumpCPMemory {
        /// Output file where the codeplug data will be written.
        path: std::path::PathBuf,
    },

    /// Write a codeplug image back into codeplug memory.
    WriteCPMemory {
        /// Input codeplug image file.
        path: std::path::PathBuf,
    },

    /// Dump a codeplug memory image to a file via firmware update mode.
    FwDumpCPMemory {
        /// Output file where the codeplug data will be written.
        path: std::path::PathBuf,
    },

    /// Write a codeplug image bacl to the codeplug memowry via firmware update mode.
    FwWriteCPMemory {
        /// Input codeplug image file.
        path: std::path::PathBuf,
    },

    /// Print out all sections contained within the codeplug.
    PrintSections {
        /// Path to codeplug image. If not specified the codeplug is read
        /// directly from the radio.
        codeplug_image: Option<std::path::PathBuf>,
    },

    /// Parse the codeplug and print out a textual representation of the
    /// parsable sections.
    PrintCodeplug {
        /// Path to codeplug image. If not specified the codeplug is read
        /// directly from the radio.
        codeplug_image: Option<std::path::PathBuf>,
    },

    /// Diesect the codeplug, writing out earch section's data elements to a
    /// file.
    Disect {
        /// Directory where codeplug elements will be written.
        output_directory: std::path::PathBuf,

        /// Path to codeplug image. If not specified the codeplug is read
        /// directly from the radio.
        codeplug_image: Option<std::path::PathBuf>,
    },

    /// Verify a given codeplug, checking that various values aren't out of
    /// range and that no broken links exist.
    Verify {
        /// Path to codeplug image. If not specified the codeplug is read
        /// directly from the radio.
        codeplug_image: Option<std::path::PathBuf>,
    },

    WriteCustomCodeplug {
        /// Path to codeplug image. If not specified the codeplug is read
        /// directly from the radio.
        codeplug_image: Option<std::path::PathBuf>,

        output_file: Option<std::path::PathBuf>,
    },
}

fn pb_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {percent}% {msg}")
        .unwrap()
        .progress_chars("##-")
}

fn write_codeplug_image(path: &std::path::PathBuf, verbose: bool) -> Result<()> {
    let pb = ProgressBar::new(CPS_MEM_MAX_SZ as u64);
    pb.set_style(pb_style());
    pb.set_message("Write Codeplug (CPS)");
    let mut in_file = File::open(path).context("Could not open output file")?;
    let mut buf = vec![];

    in_file
        .read_to_end(&mut buf)
        .context("Failed to read input data")?;

    let mut radio = pb.wrap_write(Radio::new(verbose, CPSMode::new(OpenMode::Write))?);

    radio
        .write_all(&buf)
        .context("Failed to write data to radio")?;

    Ok(())
}

fn dump_codeplug_image(path: &std::path::PathBuf, verbose: bool) -> Result<()> {
    let pb = ProgressBar::new(CPS_MEM_MAX_SZ as u64);
    pb.set_style(pb_style());
    pb.set_message("Read Codeplug (CPS)");
    let mut radio = pb.wrap_read(Radio::new(verbose, CPSMode::new(OpenMode::Read))?);
    let mut out_file = File::create(path).context("Could not open output file")?;
    let mut buf = vec![];

    radio
        .read_to_end(&mut buf)
        .context("Failed to read data from radio")?;

    out_file
        .write_all(&buf)
        .context("Could not write data to output file")?;

    Ok(())
}

fn fw_write_codeplug_image(path: &std::path::PathBuf, verbose: bool) -> Result<()> {
    let pb = ProgressBar::new(CPS_MEM_MAX_SZ as u64);
    pb.set_style(pb_style());
    pb.set_message("Write Codeplug (FW)");
    let mut in_file = File::open(path).context("Could not open output file")?;
    let mut buf = vec![];

    in_file
        .read_to_end(&mut buf)
        .context("Failed to read input data")?;

    let mut radio = pb.wrap_write(Radio::new(verbose, FirmwareMode::new())?);

    radio
        .write_all(&buf)
        .context("Failed to write data to radio")?;

    Ok(())
}

fn fw_dump_codeplug_image(path: &std::path::PathBuf, verbose: bool) -> Result<()> {
    let mut out_file = File::create(path).context("Could not open output file")?;
    let mut buf = vec![];
    let pb = ProgressBar::new(CPS_MEM_MAX_SZ as u64);
    pb.set_message("Read Codeplug (FW)");
    pb.set_style(pb_style());
    let mut radio = pb.wrap_read(Radio::new(verbose, FirmwareMode::new())?);

    radio
        .read_to_end(&mut buf)
        .context("Failed to read data from radio")?;

    out_file
        .write_all(&buf)
        .context("Could not write data to output file")?;

    Ok(())
}

pub fn print_sections(path: &Option<std::path::PathBuf>, verbose: bool) -> Result<()> {
    let mut src = get_source(path, verbose)?;
    let sections = Section::load_sections(&mut src).context("Failed to parse sections")?;

    let mut table = Table::new();

    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header([
        "Address",
        "Type",
        "Capacity",
        "Elements in Use",
        "Byte Size",
        "Unk1",
        "Unk2",
    ]);

    for (_, section) in sections
        .iter()
        .sorted_by(|a, b| Ord::cmp(&a.1.header.section_type, &b.1.header.section_type))
    {
        table.add_row([
            format!("0x{:X}", section.addr),
            format!("0x{:X}", section.header.section_type),
            format!("0x{:X}", section.header.capacity),
            format!("0x{:X}", section.header.elements_in_use),
            format!("0x{:X}", section.header.byte_size),
            format!("0x{:X}", section.header.unk1),
            format!("0x{:X}", section.header.unk2),
        ]);
    }

    println!("{table}");

    Ok(())
}

pub fn print_codeplug(path: &Option<std::path::PathBuf>, verbose: bool) -> Result<()> {
    let mut src = get_source(path, verbose)?;
    let codeplug = Codeplug::read_codeplug(&mut src).context("Failed to read codeplug")?;

    println!("{}", codeplug);

    Ok(())
}

fn disect_codeplug(
    codeplug_image: &Option<std::path::PathBuf>,
    output_directory: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    let mut src = get_source(codeplug_image, verbose)?;
    let sections = Section::load_sections(&mut src)?;

    for (_, sec) in sections.iter() {
        let sec_kind = sec.header.section_type;
        let sec_dir = output_directory.join(format!("0x{sec_kind:04X}"));
        std::fs::create_dir(&sec_dir)?;

        for idx in 0..sec.header.elements_in_use {
            let mut out_file = File::create(sec_dir.join(format!("{idx:04}")))?;
            out_file.write_all(sec.get_data_chunk(idx)?)?;
        }
    }

    Ok(())
}

fn verify_codeplug(codeplug_image: &Option<std::path::PathBuf>, verbose: bool) -> Result<()> {
    let mut src = get_source(codeplug_image, verbose)?;
    let cp = Codeplug::read_codeplug(&mut src)?;

    cp.verify()?;

    println!("Verification finished.");

    Ok(())
}

fn write_custom_codeplug(
    codeplug_image: &Option<std::path::PathBuf>,
    output_file: &Option<std::path::PathBuf>,
    verbose: bool,
) -> Result<()> {
    let mut src = get_source(codeplug_image, verbose)?;
    let mut cp = Codeplug::read_codeplug(&mut src)?;

    drop(src);

    let mut dst = get_sink(output_file, verbose).context("Could not open output")?;

    cp.mutate_cp();

    cp.write_codeplug(&mut dst)
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::DumpCPMemory { path } => dump_codeplug_image(&path, args.verbose),
        Commands::WriteCPMemory { path } => write_codeplug_image(&path, args.verbose),
        Commands::FwDumpCPMemory { path } => fw_dump_codeplug_image(&path, args.verbose),
        Commands::FwWriteCPMemory { path } => fw_write_codeplug_image(&path, args.verbose),
        Commands::PrintSections { codeplug_image } => print_sections(&codeplug_image, args.verbose),
        Commands::PrintCodeplug { codeplug_image } => print_codeplug(&codeplug_image, args.verbose),
        Commands::Disect {
            output_directory,
            codeplug_image,
        } => disect_codeplug(&codeplug_image, &output_directory, args.verbose),
        Commands::Verify { codeplug_image } => verify_codeplug(&codeplug_image, args.verbose),
        Commands::WriteCustomCodeplug {
            codeplug_image,
            output_file,
        } => write_custom_codeplug(&codeplug_image, &output_file, args.verbose),
    }
}
