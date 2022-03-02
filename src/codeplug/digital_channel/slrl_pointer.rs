#[cfg(test)]
use proptest::strategy::Strategy;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[cfg(test)]
fn ptr_strategy() -> impl Strategy<Value = u8> {
    0..254u8
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum SLRLPointer {
    None,
    #[cfg_attr(
        test,
        proptest(strategy = "ptr_strategy().prop_map(SLRLPointer::ScanList)")
    )]
    ScanList(u8),
    #[cfg_attr(
        test,
        proptest(strategy = "ptr_strategy().prop_map(SLRLPointer::RoamList)")
    )]
    RoamList(u8),
}
