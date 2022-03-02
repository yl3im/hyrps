#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CTCSSTailRevertPhase {
    Rad120,
    Rad180,
}

impl From<u8> for CTCSSTailRevertPhase {
    fn from(v: u8) -> Self {
        match v & 0x20 {
            0 => CTCSSTailRevertPhase::Rad120,
            0x20 => CTCSSTailRevertPhase::Rad180,
            _ => unreachable!(),
        }
    }
}
