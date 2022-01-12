pub use cenum::cenum;

#[cenum]
#[repr(u8)]
enum TestEnumBasic {
    Value1,
    Value2,
    Value3,
}

#[test]
fn can_serialize() {
    assert_eq!(TestEnumBasic::Value2.into_primitive(), 1);
}

#[test]
fn can_deserialize() {
    assert_eq!(
        TestEnumBasic::from_primitive(1),
        Some(TestEnumBasic::Value2)
    );
}

#[test]
fn can_check() {
    assert_eq!(TestEnumBasic::from_primitive(9u8).is_some(), false);
    assert_eq!(TestEnumBasic::from_primitive(2).is_some(), true);
}

#[cenum]
enum TestEnumDeterminant {
    Value1,
    Value2 = 7,
    Value3,
}

#[test]
fn can_serialize_determinant() {
    assert_eq!(TestEnumDeterminant::Value3.into_primitive(), 8);
}

#[test]
fn can_deserialize_determinant() {
    assert_eq!(
        TestEnumDeterminant::from_primitive(8),
        Some(TestEnumDeterminant::Value3)
    );
}

#[test]
fn can_check_determinant() {
    assert_eq!(TestEnumDeterminant::from_primitive(8).is_some(), true);
    assert_eq!(TestEnumDeterminant::from_primitive(0u32).is_some(), true);
    assert_eq!(TestEnumDeterminant::from_primitive(3).is_some(), false);
    assert_eq!(TestEnumDeterminant::from_primitive(9).is_some(), false);
}

#[cenum]
#[repr(i32)]
enum TestEnumNegative {
    Value1 = -3,
    Value2 = -2,
    Value3 = -1,
    Value4 = 7,
}

#[test]
fn enum_negative() {
    assert_eq!(TestEnumNegative::from_primitive(-3i32).is_some(), true);
    assert_eq!(TestEnumNegative::from_primitive(7).is_some(), true);
    assert_eq!(
        TestEnumNegative::from_primitive(-3),
        Some(TestEnumNegative::Value1)
    );
    assert_eq!(TestEnumNegative::Value1.into_primitive(), -3);
}
