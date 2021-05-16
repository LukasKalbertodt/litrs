use crate::{
    Lit, Error, Integer, IntegerType as Ty, IntegerBase, IntegerBase::*,
    test_util::assert_parse_ok_eq,
};


macro_rules! assert_int_parse {
    ($input:literal, $v:literal, $expected:expr) => {
        assert_parse_ok_eq($input, Lit::parse($input), Lit::Integer($expected), "Lit::parse");
        assert_parse_ok_eq($input, Integer::parse($input), $expected, "Integer::parse");
        assert_eq!(Integer::parse($input).unwrap().value().expect("unexpected overflow"), $v);
    };
}

macro_rules! assert_err {
    ($input:literal) => {
        assert!(Lit::parse($input).is_err());
        assert!(Integer::parse($input).is_err());
    };
}


fn int(base: IntegerBase, main_part: &str, type_suffix: Option<Ty>) -> Integer {
    Integer {
        base,
        main_part,
        type_suffix,
    }
}

#[test]
fn parse_decimal() {
    assert_int_parse!("0", 0, int(Decimal, "0", None));
    assert_int_parse!("1", 1, int(Decimal, "1", None));
    assert_int_parse!("8", 8, int(Decimal, "8", None));
    assert_int_parse!("9", 9, int(Decimal, "9", None));
    assert_int_parse!("10", 10, int(Decimal, "10", None));
    assert_int_parse!("11", 11, int(Decimal, "11", None));
    assert_int_parse!("123456789", 123456789, int(Decimal, "123456789", None));

    assert_int_parse!("05", 5, int(Decimal, "05", None));
    assert_int_parse!("00005", 5, int(Decimal, "00005", None));
    assert_int_parse!("0123456789", 123456789, int(Decimal, "0123456789", None));

    assert_int_parse!("123_456_789", 123_456_789, int(Decimal, "123_456_789", None));
    assert_int_parse!("0___4", 4, int(Decimal, "0___4", None));
    assert_int_parse!("0___4_3", 43, int(Decimal, "0___4_3", None));
    assert_int_parse!("0___4_3", 43, int(Decimal, "0___4_3", None));
    assert_int_parse!("123___________", 123, int(Decimal, "123___________", None));

    assert_int_parse!(
        "340282366920938463463374607431768211455",
        340282366920938463463374607431768211455u128,
        int(Decimal, "340282366920938463463374607431768211455", None)
    );
    assert_int_parse!(
        "340_282_366_920_938_463_463_374_607_431_768_211_455",
        340282366920938463463374607431768211455u128,
        int(Decimal, "340_282_366_920_938_463_463_374_607_431_768_211_455", None)
    );
    assert_int_parse!(
        "3_40_282_3669_20938_463463_3746074_31768211_455___",
        340282366920938463463374607431768211455u128,
        int(Decimal, "3_40_282_3669_20938_463463_3746074_31768211_455___", None)
    );
}

#[test]
fn overflow() {
    let inputs = [
        "340282366920938463463374607431768211456",
        "340282366920938463463374607431768211456u128",
        "340282366920938463463374607431768211457",
        "3_40_282_3669_20938_463463_3746074_31768211_456___",
        "3_40_282_3669_20938_463463_3746074_31768211_455___1",
        "3_40_282_3669_20938_463463_3746074_31768211_455___0u128",
        "3402823669209384634633746074317682114570",
    ];

    for input in &inputs {
        let lit = Integer::parse(input).expect("failed to parse");
        assert_eq!(lit.base, Decimal);
        assert!(lit.value().is_none());
    }
}

#[test]
fn parse_err() {
    assert_err!("");
    assert_err!("a");
    assert_err!(";");
    assert_err!("0;");
    assert_err!("0a");
    assert_err!("0b");
    assert_err!("0z");
    assert_err!(" 0");
    assert_err!("0 ");

    assert_eq!(Integer::parse("0x_"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x__"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x________"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x_i8"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x_u8"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x_isize"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0x_usize"), Err(Error::NoValidDigits));

    assert_eq!(Integer::parse("0o_"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0o__"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0o________"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0o_i32"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0o_u32"), Err(Error::NoValidDigits));

    assert_eq!(Integer::parse("0b_"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0b__"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0b________"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0b_i128"), Err(Error::NoValidDigits));
    assert_eq!(Integer::parse("0b_u128"), Err(Error::NoValidDigits));
}
