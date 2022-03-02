use super::codeplug::cp_data::RawCPData;
use proptest::prelude::*;
use std::io::Seek;

pub fn check_serde<T: RawCPData + std::fmt::Debug + std::cmp::PartialEq>(
    obj: &T,
) -> Result<(), TestCaseError> {
    let vec = Vec::new();
    let mut cursor = std::io::Cursor::new(vec);

    obj.store(&mut cursor).unwrap();

    cursor.rewind().unwrap();

    let obj2 = T::load(&mut cursor).unwrap();

    prop_assert_eq!(obj, &obj2);

    Ok(())
}
