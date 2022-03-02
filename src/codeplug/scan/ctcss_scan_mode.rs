use std::convert::TryFrom;
use std::io::Error;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CTCSSScanMode {
    Disabled,
    NonPriorityChannel,
    PriorityChannel,
    PriorityAndNonPriorityChannel,
}

impl TryFrom<u16> for CTCSSScanMode {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match (value >> 2) & 3 {
            0 => Ok(CTCSSScanMode::Disabled),
            1 => Ok(CTCSSScanMode::NonPriorityChannel),
            2 => Ok(CTCSSScanMode::PriorityChannel),
            3 => Ok(CTCSSScanMode::PriorityAndNonPriorityChannel),
            _ => unreachable!(),
        }
    }
}
