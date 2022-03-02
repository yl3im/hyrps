#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum PowerLevel {
    High,
    Low,
}

impl From<u8> for PowerLevel {
    fn from(v: u8) -> Self {
        match v & 0x4 {
            0 => PowerLevel::High,
            4 => PowerLevel::Low,
            _ => unreachable!(),
        }
    }
}
