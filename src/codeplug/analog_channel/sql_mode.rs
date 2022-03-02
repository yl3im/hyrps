use num_enum::TryFromPrimitive;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum SqlMode {
    Carrier = 0,
    CtcssCdcss = 1,
    OptSignaling = 2,
    CtcssCdcssAndOptSig = 3,
    CtcssCdcssOrOptSig = 4,
}
