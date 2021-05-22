use crate::{
    Lit, Error,
    test_util::assert_parse_ok_eq,
};
use super::{Float, FloatType};


// ===== Utility functions =======================================================================

/// Helper macro to check parsing a float.
///
/// This macro contains quite a bit of logic itself (which can be buggy of
/// course), so we have a few test functions below to test a bunch of cases
/// manually.
macro_rules! check {
    ($intpart:literal $fracpart:literal $exppart:literal $suffix:tt) => {
        let input = concat!($intpart, $fracpart, $exppart, check!(@stringify_suffix $suffix));
        let expected_float = Float {
            number_part: concat!($intpart, $fracpart, $exppart),
            end_integer_part: $intpart.len(),
            end_fractional_part: $intpart.len() + $fracpart.len(),
            type_suffix: check!(@ty $suffix),
        };

        assert_parse_ok_eq(input, Float::parse(input), expected_float.clone(), "Float::parse");
        assert_parse_ok_eq(input, Lit::parse(input), Lit::Float(expected_float), "Lit::parse");

    };
    (@ty f32) => { Some(FloatType::F32) };
    (@ty f64) => { Some(FloatType::F64) };
    (@ty -) => { None };
    (@stringify_suffix -) => { "" };
    (@stringify_suffix $suffix:ident) => { stringify!($suffix) };
}


// ===== Actual tests ===========================================================================

#[test]
fn manual_without_suffix() -> Result<(), Error> {
    let f = Float::parse("3.14")?;
    assert_eq!(f.number_part(), "3.14");
    assert_eq!(f.integer_part(), "3");
    assert_eq!(f.fractional_part(), Some("14"));
    assert_eq!(f.exponent_part(), "");
    assert_eq!(f.type_suffix(), None);

    let f = Float::parse("9.")?;
    assert_eq!(f.number_part(), "9.");
    assert_eq!(f.integer_part(), "9");
    assert_eq!(f.fractional_part(), Some(""));
    assert_eq!(f.exponent_part(), "");
    assert_eq!(f.type_suffix(), None);

    let f = Float::parse("8e1")?;
    assert_eq!(f.number_part(), "8e1");
    assert_eq!(f.integer_part(), "8");
    assert_eq!(f.fractional_part(), None);
    assert_eq!(f.exponent_part(), "e1");
    assert_eq!(f.type_suffix(), None);

    let f = Float::parse("8E3")?;
    assert_eq!(f.number_part(), "8E3");
    assert_eq!(f.integer_part(), "8");
    assert_eq!(f.fractional_part(), None);
    assert_eq!(f.exponent_part(), "E3");
    assert_eq!(f.type_suffix(), None);

    let f = Float::parse("8_7_6.1_23e15")?;
    assert_eq!(f.number_part(), "8_7_6.1_23e15");
    assert_eq!(f.integer_part(), "8_7_6");
    assert_eq!(f.fractional_part(), Some("1_23"));
    assert_eq!(f.exponent_part(), "e15");
    assert_eq!(f.type_suffix(), None);

    let f = Float::parse("8.2e-_04_9")?;
    assert_eq!(f.number_part(), "8.2e-_04_9");
    assert_eq!(f.integer_part(), "8");
    assert_eq!(f.fractional_part(), Some("2"));
    assert_eq!(f.exponent_part(), "e-_04_9");
    assert_eq!(f.type_suffix(), None);

    Ok(())
}

#[test]
fn manual_with_suffix() -> Result<(), Error> {
    let f = Float::parse("3.14f32")?;
    assert_eq!(f.number_part(), "3.14");
    assert_eq!(f.integer_part(), "3");
    assert_eq!(f.fractional_part(), Some("14"));
    assert_eq!(f.exponent_part(), "");
    assert_eq!(f.type_suffix(), Some(FloatType::F32));

    let f = Float::parse("8e1f64")?;
    assert_eq!(f.number_part(), "8e1");
    assert_eq!(f.integer_part(), "8");
    assert_eq!(f.fractional_part(), None);
    assert_eq!(f.exponent_part(), "e1");
    assert_eq!(f.type_suffix(), Some(FloatType::F64));

    let f = Float::parse("8_7_6.1_23e15f32")?;
    assert_eq!(f.number_part(), "8_7_6.1_23e15");
    assert_eq!(f.integer_part(), "8_7_6");
    assert_eq!(f.fractional_part(), Some("1_23"));
    assert_eq!(f.exponent_part(), "e15");
    assert_eq!(f.type_suffix(), Some(FloatType::F32));

    let f = Float::parse("8.2e-_04_9f64")?;
    assert_eq!(f.number_part(), "8.2e-_04_9");
    assert_eq!(f.integer_part(), "8");
    assert_eq!(f.fractional_part(), Some("2"));
    assert_eq!(f.exponent_part(), "e-_04_9");
    assert_eq!(f.type_suffix(), Some(FloatType::F64));

    Ok(())
}

#[test]
fn simple() {
    check!("3" ".14" "" f32);
    check!("3" ".14" "" -);
}


// 3._4
// 1.e4
// 1e
// .5
//
