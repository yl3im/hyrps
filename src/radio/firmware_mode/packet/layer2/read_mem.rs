use super::FwMemory;

pub struct FwReadMemoryReq {
    pub mem: FwMemory,
    pub addr: u32,
    pub len: usize,
}

impl
