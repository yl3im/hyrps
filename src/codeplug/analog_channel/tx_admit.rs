use num_enum::TryFromPrimitive;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum TxAdmit {
    Always = 0,
    ChannelFree = 1,
    CTCSSCorrect = 2,
    CTCSSIncorrect = 3,
}
