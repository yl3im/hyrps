#[cfg(test)]
use proptest::strategy::Strategy;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[cfg(test)]
fn idx_strategy() -> impl Strategy<Value = u16> {
    1..(u16::MAX - 1)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum RrsRevertCh {
    ChSelf,
    None,
    #[cfg_attr(test, proptest(strategy = "idx_strategy().prop_map(RrsRevertCh::Idx)"))]
    Idx(u16),
}

impl From<u16> for RrsRevertCh {
    fn from(v: u16) -> Self {
        match v {
            0 => RrsRevertCh::ChSelf,
            0xffff => RrsRevertCh::None,
            _ => RrsRevertCh::Idx(v),
        }
    }
}

impl From<RrsRevertCh> for u16 {
    fn from(v: RrsRevertCh) -> Self {
        match v {
            RrsRevertCh::ChSelf => 0,
            RrsRevertCh::None => 0xffff,
            RrsRevertCh::Idx(v) => v,
        }
    }
}
