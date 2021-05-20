use std::fmt::{Debug, Display};
use crate::{
    Lit, Error, Integer, IntegerType as Ty, IntegerBase, IntegerBase::*,
    test_util::assert_parse_ok_eq,
};

use super::FromIntLiteral;


// ===== Utility functions =======================================================================

#[track_caller]
fn assert_err(input: &str) {
    assert!(Lit::parse(input).is_err());
    assert!(Integer::parse(input).is_err());
}

#[track_caller]
fn check<T: FromIntLiteral + PartialEq + Debug + Display>(
    input: &str,
    value: T,
    base: IntegerBase,
    main_part: &str,
    type_suffix: Option<Ty>,
) {
    let expected_integer = Integer { base, main_part, type_suffix };
    assert_parse_ok_eq(input, Integer::parse(input), expected_integer.clone(), "Integer::parse");
    assert_parse_ok_eq(input, Lit::parse(input), Lit::Integer(expected_integer), "Lit::parse");

    let actual_value = Integer::parse(input)
        .unwrap()
        .value::<T>()
        .unwrap_or_else(|| panic!("unexpected overflow in `Integer::value` for `{}`", input));
    if actual_value != value {
        panic!(
            "Parsing int literal `{}` should give value `{}`, but actually resulted in `{}`",
            input,
            value,
            actual_value,
        );
    }
}


// ===== Actual tests ===========================================================================

#[test]
fn parse_decimal() {
    check("0", 0u128, Decimal, "0", None);
    check("1", 1u8, Decimal, "1", None);
    check("8", 8u16, Decimal, "8", None);
    check("9", 9u32, Decimal, "9", None);
    check("10", 10u64, Decimal, "10", None);
    check("11", 11i8, Decimal, "11", None);
    check("123456789", 123456789i128, Decimal, "123456789", None);

    check("05", 5i16, Decimal, "05", None);
    check("00005", 5i32, Decimal, "00005", None);
    check("0123456789", 123456789i64, Decimal, "0123456789", None);

    check("123_456_789", 123_456_789, Decimal, "123_456_789", None);
    check("0___4", 4, Decimal, "0___4", None);
    check("0___4_3", 43, Decimal, "0___4_3", None);
    check("0___4_3", 43, Decimal, "0___4_3", None);
    check("123___________", 123, Decimal, "123___________", None);

    check(
        "340282366920938463463374607431768211455",
        340282366920938463463374607431768211455u128,
        Decimal,
        "340282366920938463463374607431768211455",
        None,
    );
    check(
        "340_282_366_920_938_463_463_374_607_431_768_211_455",
        340282366920938463463374607431768211455u128,
        Decimal,
        "340_282_366_920_938_463_463_374_607_431_768_211_455",
        None,
    );
    check(
        "3_40_282_3669_20938_463463_3746074_31768211_455___",
        340282366920938463463374607431768211455u128,
        Decimal,
        "3_40_282_3669_20938_463463_3746074_31768211_455___",
        None,
    );
}

#[test]
fn suffixes() {
    [
        ("123i8", Ty::I8),
        ("123i16", Ty::I16),
        ("123i32", Ty::I32),
        ("123i64", Ty::I64),
        ("123i128", Ty::I128),
        ("123u8", Ty::U8),
        ("123u16", Ty::U16),
        ("123u32", Ty::U32),
        ("123u64", Ty::U64),
        ("123u128", Ty::U128),
    ].iter().for_each(|&(s, ty)| {
        assert_eq!(Integer::parse(s).unwrap().type_suffix(), Some(ty));
    });
}

#[test]
fn overflow_u128() {
    let inputs = [
        "340282366920938463463374607431768211456",
        "0x100000000000000000000000000000000",
        "0o4000000000000000000000000000000000000000000",
        "0b1000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000",
        "340282366920938463463374607431768211456u128",
        "340282366920938463463374607431768211457",
        "3_40_282_3669_20938_463463_3746074_31768211_456___",
        "3_40_282_3669_20938_463463_3746074_31768211_455___1",
        "3_40_282_3669_20938_463463_3746074_31768211_455___0u128",
        "3402823669209384634633746074317682114570",
    ];

    for input in &inputs {
        let lit = Integer::parse(input).expect("failed to parse");
        assert!(lit.value::<u128>().is_none());
    }
}

#[test]
fn overflow_u8() {
    let inputs = [
        "256", "0x100", "0o400", "0b100000000",
        "257", "0x101", "0o401", "0b100000001",
        "300",
        "1548",
        "2548985",
        "256u128",
        "256u8",
        "2_5_6",
        "256_____1",
        "256__",
    ];

    for input in &inputs {
        let lit = Integer::parse(input).expect("failed to parse");
        assert!(lit.value::<u8>().is_none());
    }
}

#[test]
fn parse_err() {
    [
        "",
        "a",
        ";",
        "0;",
        "0a",
        "0b",
        "0z",
        " 0",
        "0 ",
        "0a3",
        "0b3",
        "0z3",
    ].iter().for_each(|s| assert_err(s));
}

#[test]
fn no_valid_digits() {
    [
        "0x_",
        "0x__",
        "0x________",
        "0x_i8",
        "0x_u8",
        "0x_isize",
        "0x_usize",

        "0o_",
        "0o__",
        "0o________",
        "0o_i32",
        "0o_u32",

        "0b_",
        "0b__",
        "0b________",
        "0b_i128",
        "0b_u128",
    ].iter().for_each(|s| assert_eq!(Integer::parse(s), Err(Error::NoValidDigits)));
}

#[test]
fn invalid_suffix() {
    [
        "5u7", "5u9", "5u0", "33u12", "84u17", "99u80", "1234uu16",
        "5i7", "5i9", "5i0", "33i12", "84i17", "99i80", "1234ii16",
        "0ui32", "1iu32",
        "54321a64",
        "54321b64",
        "54321x64",
        "54321o64",
    ].iter().for_each(|s| assert_err(s));
}
