use crate::codeplug::{cp_data::RawCPData, Codeplug};

pub trait RawPointer: RawCPData {
    fn sz() -> usize;

    fn verify(&self, cp: &Codeplug) -> anyhow::Result<()>;
}
