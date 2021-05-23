use std::fmt::{Debug, Display};
use crate::{
    Lit, Integer, IntegerType as Ty, IntegerBase, IntegerBase::*,
    test_util::assert_parse_ok_eq,
};

use super::FromIntLiteral;


// ===== Utility functions =======================================================================

#[track_caller]
fn assert_err(input: &str) {
    if Lit::parse(input).is_ok() {
        panic!("Parsing '{}' with `Lit::parse` should fail, but it didn't!", input);
    }
    if Integer::parse(input).is_ok() {
        panic!("Parsing '{}' with `Integer::parse` should fail, but it didn't!", input);
    }
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
fn parse_binary() {
    check("0b0", 0b0, Binary, "0", None);
    check("0b000", 0b000, Binary, "000", None);
    check("0b1", 0b1, Binary, "1", None);
    check("0b01", 0b01, Binary, "01", None);
    check("0b101010", 0b101010, Binary, "101010", None);
    check("0b10_10_10", 0b10_10_10, Binary, "10_10_10", None);
    check("0b01101110____", 0b01101110____, Binary, "01101110____", None);

    check("0b10010u8", 0b10010u8, Binary, "10010", Some(Ty::U8));
    check("0b10010i8", 0b10010u8, Binary, "10010", Some(Ty::I8));
    check("0b10010u64", 0b10010u64, Binary, "10010", Some(Ty::U64));
    check("0b10010i64", 0b10010u64, Binary, "10010", Some(Ty::I64));
    check(
        "0b1011001_00110000_00101000_10100101u32",
        0b1011001_00110000_00101000_10100101u32,
        Binary,
        "1011001_00110000_00101000_10100101",
        Some(Ty::U32),
    );
}

#[test]
fn parse_octal() {
    check("0o0", 0o0, Octal, "0", None);
    check("0o1", 0o1, Octal, "1", None);
    check("0o6", 0o6, Octal, "6", None);
    check("0o7", 0o7, Octal, "7", None);
    check("0o17", 0o17, Octal, "17", None);
    check("0o123", 0o123, Octal, "123", None);
    check("0o7654321", 0o7654321, Octal, "7654321", None);
    check("0o7_53_1", 0o7_53_1, Octal, "7_53_1", None);
    check("0o66_", 0o66_, Octal, "66_", None);

    check("0o755u16", 0o755u16, Octal, "755", Some(Ty::U16));
    check("0o755i128", 0o755i128, Octal, "755", Some(Ty::I128));
}

#[test]
fn parse_hexadecimal() {
    check("0x0", 0x0, Hexadecimal, "0", None);
    check("0x1", 0x1, Hexadecimal, "1", None);
    check("0x9", 0x9, Hexadecimal, "9", None);

    check("0xa", 0xa, Hexadecimal, "a", None);
    check("0xf", 0xf, Hexadecimal, "f", None);
    check("0x17", 0x17, Hexadecimal, "17", None);
    check("0x1b", 0x1b, Hexadecimal, "1b", None);
    check("0x123", 0x123, Hexadecimal, "123", None);
    check("0xace", 0xace, Hexadecimal, "ace", None);
    check("0xfdb971", 0xfdb971, Hexadecimal, "fdb971", None);
    check("0xa_54_f", 0xa_54_f, Hexadecimal, "a_54_f", None);
    check("0x6d_", 0x6d_, Hexadecimal, "6d_", None);

    check("0xA", 0xA, Hexadecimal, "A", None);
    check("0xF", 0xF, Hexadecimal, "F", None);
    check("0x17", 0x17, Hexadecimal, "17", None);
    check("0x1B", 0x1B, Hexadecimal, "1B", None);
    check("0x123", 0x123, Hexadecimal, "123", None);
    check("0xACE", 0xACE, Hexadecimal, "ACE", None);
    check("0xFDB971", 0xFDB971, Hexadecimal, "FDB971", None);
    check("0xA_54_F", 0xA_54_F, Hexadecimal, "A_54_F", None);
    check("0x6D_", 0x6D_, Hexadecimal, "6D_", None);

    check("0xFdB97a1", 0xFdB97a1, Hexadecimal, "FdB97a1", None);
    check("0xfdB97A1", 0xfdB97A1, Hexadecimal, "fdB97A1", None);

    check("0x40u16", 0x40u16, Hexadecimal, "40", Some(Ty::U16));
    check("0xffi128", 0xffi128, Hexadecimal, "ff", Some(Ty::I128));
}

#[test]
fn starting_underscore() {
    check("0b_1", 1, Binary, "_1", None);
    check("0b_010i16", 0b_010, Binary, "_010", Some(Ty::I16));

    check("0o_5", 5, Octal, "_5", None);
    check("0o_750u128", 0o_750u128, Octal, "_750", Some(Ty::U128));

    check("0x_c", 0xc, Hexadecimal, "_c", None);
    check("0x_cf3i8", 0x_cf3, Hexadecimal, "_cf3", Some(Ty::I8));
}

#[test]
fn parse_overflowing_just_fine() {
    check("256u8", 256u16, Decimal, "256", Some(Ty::U8));
    check("123_456_789u8", 123_456_789u32, Decimal, "123_456_789", Some(Ty::U8));
    check("123_456_789u16", 123_456_789u32, Decimal, "123_456_789", Some(Ty::U16));

    check("123_123_456_789u8", 123_123_456_789u64, Decimal, "123_123_456_789", Some(Ty::U8));
    check("123_123_456_789u16", 123_123_456_789u64, Decimal, "123_123_456_789", Some(Ty::U16));
    check("123_123_456_789u32", 123_123_456_789u64, Decimal, "123_123_456_789", Some(Ty::U32));
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

    for &input in &inputs {
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

    for &input in &inputs {
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
        "_",
        "_3",
    ].iter().for_each(|&s| assert_err(s));
}

#[test]
fn invalid_digits() {
    // TODO: emit better error
    [
        "0b10201",
        "0b9",
        "0b07",
        "0b0a", "0b0A",
        "0b0f", "0b0F",

        "0o12380",
        "0o192",
        "0o7a_", "0o7A_",
        "0o7f_0", "0o7F_0",

        "12a3",
        "12f3",
        "12f_",
        "12F_",
        "a_123",
        "B_123",

        "0x8cg",
        "0x8cG",
        "0x8ch_",
        "0x8cH_",
        "0x8czu16",
    ].iter().for_each(|&s| assert_err(s));
}

#[test]
fn no_valid_digits() {
    assert_err!(Integer::parse("0x_"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0x__"), NoValidDigits, 2..4);
    assert_err!(Integer::parse("0x________"), NoValidDigits, 2..10);
    assert_err!(Integer::parse("0x_i8"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0x_u8"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0x_isize"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0x_usize"), NoValidDigits, 2..3);

    assert_err!(Integer::parse("0o_"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0o__"), NoValidDigits, 2..4);
    assert_err!(Integer::parse("0o________"), NoValidDigits, 2..10);
    assert_err!(Integer::parse("0o_i32"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0o_u32"), NoValidDigits, 2..3);

    assert_err!(Integer::parse("0b_"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0b__"), NoValidDigits, 2..4);
    assert_err!(Integer::parse("0b________"), NoValidDigits, 2..10);
    assert_err!(Integer::parse("0b_i128"), NoValidDigits, 2..3);
    assert_err!(Integer::parse("0b_u128"), NoValidDigits, 2..3);
}

#[test]
fn invalid_suffix() {
    assert_err!(Integer::parse("5u7"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("5u9"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("5u0"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("33u12"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("84u17"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("99u80"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("1234uu16"), InvalidIntegerTypeSuffix, 4..8);

    assert_err!(Integer::parse("5i7"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("5i9"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("5i0"), InvalidIntegerTypeSuffix, 1..3);
    assert_err!(Integer::parse("33i12"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("84i17"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("99i80"), InvalidIntegerTypeSuffix, 2..5);
    assert_err!(Integer::parse("1234ii16"), InvalidIntegerTypeSuffix, 4..8);

    assert_err!(Integer::parse("0ui32"), InvalidIntegerTypeSuffix, 1..5);
    assert_err!(Integer::parse("1iu32"), InvalidIntegerTypeSuffix, 1..5);
    assert_err!(Integer::parse("54321a64"), InvalidIntegerTypeSuffix, 5..8);
    assert_err!(Integer::parse("54321b64"), InvalidIntegerTypeSuffix, 5..8);
    assert_err!(Integer::parse("54321x64"), InvalidIntegerTypeSuffix, 5..8);
    assert_err!(Integer::parse("54321o64"), InvalidIntegerTypeSuffix, 5..8);
}
