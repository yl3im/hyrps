#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Timeslot {
    Slot1,
    Slot2,
    PseudoTrunk,
}

impl From<u8> for Timeslot {
    fn from(v: u8) -> Self {
        match v {
            0 => Timeslot::Slot1,
            1 => Timeslot::Slot2,
            3 => Timeslot::PseudoTrunk,
            _ => panic!("Unexpected timeslot value: {}", v),
        }
    }
}

impl From<Timeslot> for u8 {
    fn from(v: Timeslot) -> Self {
        match v {
            Timeslot::Slot1 => 0,
            Timeslot::Slot2 => 1,
            Timeslot::PseudoTrunk => 3,
        }
    }
}
