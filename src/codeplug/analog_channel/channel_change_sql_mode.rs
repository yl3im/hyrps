use num_enum::TryFromPrimitive;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ChannelChangeSqlMode {
    RxSQLMode = 0,
    MonitorSqlMode = 1,
}
