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

#[track_caller]
fn assert_err(input: &str) {
    if Lit::parse(input).is_ok() {
        panic!("Parsing '{}' with `Lit::parse` should fail, but it didn't!", input);
    }
    if Float::parse(input).is_ok() {
        panic!("Parsing '{}' with `Float::parse` should fail, but it didn't!", input);
    }
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
    [
        "",
        ".",
        "+",
        "-",
        "e",
        "e8",
        "0e",
        "f32",
        "foo",

        "inf",
        "nan",
        "NaN",
        "NAN",

        "_2.7",
        "0x44.5",
        "1e",
        ".5",
        "1.e4",
        "3._4",
        "12345._987",
        "46._",
        "46.f32",
        "46.e3",
        "46._e3",
        "46.e3f64",
        "23.4e_",
        "23E___f32",
        "7f23",
        "7f320",
        "7f64_",
        "8f649",
        "8f64f32",
        "55e3.1",

        "3.7+",
        "3.7+2",
        "3.7-",
        "3.7-2",
        "3.7e+",
        "3.7e-",
        "3.7e-+3",
        "3.7e+-3",
    ].iter().for_each(|&s| assert_err(s));
}
