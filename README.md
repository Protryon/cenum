# Cenum

## Purpose

Cenum (C + Enum) is a small Rust crate that is a spiritual successor to the `enum_primitive` crate. It uses an attribute macro over a given enum to allow low-cost conversion from a field-less enum (discriminants are allowed) to a `usize` and back again.

## Usage

```
use cenum::{ cenum, Cenum }; // exposed trait is Cenum

#[cenum]
enum MyEnum {
    Value0,
    Value1,
    Value7 = 7,
    Value8,
    Value9,
}

fn test() {
    let some_value = MyEnum::Value8;
    let serialized = some_value.to_primitive(); // to_u32/other primitive types also works
    let is_value_discriminant = MyEnum::is_discriminant(serialized);
    let deserialized = MyEnum::from_primitive(serialized); // panics if invalid value
    assert!(is_value_discriminant);
    assert_eq!(some_value, deserialized);
    assert_eq!(serialized, 8);
}
```