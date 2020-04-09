extern crate self as cenum;
pub use cenum_derive::cenum;
pub use num;

pub trait Cenum {
    fn to_primitive(&self) -> i64;
    fn from_primitive(value: i64) -> Self;
    fn is_discriminant(value: i64) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cenum]
    enum TestEnumBasic {
        Value1,
        Value2,
        Value3,
    }

    #[test]
    fn can_serialize() {
        assert_eq!(TestEnumBasic::Value2.to_primitive(), 1);
    }

    #[test]
    fn can_deserialize() {
        assert_eq!(TestEnumBasic::from_primitive(1), TestEnumBasic::Value2);
    }

    #[test]
    fn can_check() {
        assert_eq!(TestEnumBasic::is_discriminant(9), false);
        assert_eq!(TestEnumBasic::is_discriminant(2), true);
    }

    #[cenum]
    enum TestEnumDeterminant {
        Value1,
        Value2 = 7,
        Value3,
    }

    #[test]
    fn can_serialize_determinant() {
        assert_eq!(TestEnumDeterminant::Value3.to_primitive(), 8);
    }

    #[test]
    fn can_deserialize_determinant() {
        assert_eq!(
            TestEnumDeterminant::from_primitive(8),
            TestEnumDeterminant::Value3
        );
    }

    #[test]
    fn can_check_determinant() {
        assert_eq!(TestEnumDeterminant::is_discriminant(8), true);
        assert_eq!(TestEnumDeterminant::is_discriminant(0), true);
        assert_eq!(TestEnumDeterminant::is_discriminant(3), false);
        assert_eq!(TestEnumDeterminant::is_discriminant(9), false);
    }

    #[cenum]
    enum TestEnumNegative {
        Value1 = -3,
        Value2 = 7,
    }

    #[test]
    fn enum_negative() {
        assert_eq!(TestEnumNegative::is_discriminant(-3), true);
        assert_eq!(TestEnumNegative::is_discriminant(7), true);
        assert_eq!(
            TestEnumNegative::from_primitive(-3),
            TestEnumNegative::Value1
        );
        assert_eq!(TestEnumNegative::Value1.to_primitive(), -3);
    }
}
