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
    check!("3" ".14" "" -);
    check!("3" ".14" "" f32);
    check!("3" ".14" "" f64);

    check!("3" "" "" f32);
    check!("3" "" "e987654321" -);
    check!("3" "" "e987654321" f64);

    check!("42_888" ".05" "" -);
    check!("42_888" ".05" "E5___" f32);
    check!("123456789" "" "e_1" f64);
    check!("123456789" ".99" "e_1" f64);
    check!("123456789" ".99" "" f64);
    check!("123456789" ".99" "" -);

    check!("147" ".3_33" "" -);
    check!("147" ".3_33__" "E3" f64);
    check!("147" ".3_33__" "" f32);

    check!("147" ".333" "e-10" -);
    check!("147" ".333" "e-_7" f32);
    check!("147" ".333" "e+10" -);
    check!("147" ".333" "e+_7" f32);

    check!("86" "." "" -);
    check!("0" "." "" -);
    check!("0_" "." "" -);
    check!("0" ".0000001" "" -);
    check!("0" ".000_0001" "" -);

    check!("0" ".0" "e+0" -);
    check!("0" "" "E+0" -);
    check!("34" "" "e+0" -);
    check!("0" ".9182" "E+0" f32);
}

#[test]
fn parse_err() {
    assert_err!(Float::parse(""), Empty, None);
    assert_err!(Float::parse("."), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("+"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("-"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("e"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("e8"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("0e"), NoExponentDigits, 1..2);
    assert_err!(Float::parse("f32"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("foo"), DoesNotStartWithDigit, 0);

    assert_err!(Float::parse("inf"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("nan"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("NaN"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("NAN"), DoesNotStartWithDigit, 0);

    assert_err!(Float::parse("_2.7"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("0x44.5"), InvalidFloatTypeSuffix, 1..6);
    assert_err!(Float::parse("1e"), NoExponentDigits, 1..2);
    assert_err!(Float::parse(".5"), DoesNotStartWithDigit, 0);
    assert_err!(Float::parse("1.e4"), UnexpectedChar, 2);
    assert_err!(Float::parse("3._4"), UnexpectedChar, 2);
    assert_err!(Float::parse("12345._987"), UnexpectedChar, 6);
    assert_err!(Float::parse("46._"), UnexpectedChar, 3);
    assert_err!(Float::parse("46.f32"), UnexpectedChar, 3);
    assert_err!(Float::parse("46.e3"), UnexpectedChar, 3);
    assert_err!(Float::parse("46._e3"), UnexpectedChar, 3);
    assert_err!(Float::parse("46.e3f64"), UnexpectedChar, 3);
    assert_err!(Float::parse("23.4e_"), NoExponentDigits, 4..6);
    assert_err!(Float::parse("23E___f32"), NoExponentDigits, 2..6);
    assert_err!(Float::parse("7f23"), InvalidFloatTypeSuffix, 1..4);
    assert_err!(Float::parse("7f320"), InvalidFloatTypeSuffix, 1..5);
    assert_err!(Float::parse("7f64_"), InvalidFloatTypeSuffix, 1..5);
    assert_err!(Float::parse("8f649"), InvalidFloatTypeSuffix, 1..5);
    assert_err!(Float::parse("8f64f32"), InvalidFloatTypeSuffix, 1..7);
    assert_err!(Float::parse("55e3.1"), InvalidFloatTypeSuffix, 4..6);  // suboptimal

    assert_err!(Float::parse("3.7+"), InvalidFloatTypeSuffix, 3..4);
    assert_err!(Float::parse("3.7+2"), InvalidFloatTypeSuffix, 3..5);
    assert_err!(Float::parse("3.7-"), InvalidFloatTypeSuffix, 3..4);
    assert_err!(Float::parse("3.7-2"), InvalidFloatTypeSuffix, 3..5);
    assert_err!(Float::parse("3.7e+"), NoExponentDigits, 3..5);
    assert_err!(Float::parse("3.7e-"), NoExponentDigits, 3..5);
    assert_err!(Float::parse("3.7e-+3"), NoExponentDigits, 3..5);  // suboptimal
    assert_err!(Float::parse("3.7e+-3"), NoExponentDigits, 3..5);  // suboptimal
}
