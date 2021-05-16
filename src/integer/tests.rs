use crate::{
    Lit, Error, Integer, IntegerType as Ty, IntegerBase, IntegerBase::*,
    test_util::assert_parse_ok_eq,
};


macro_rules! assert_int_parse {
    ($input:literal, $expected:expr) => {
        assert_parse_ok_eq($input, Lit::parse($input), Lit::Integer($expected), "Lit::parse");
        assert_parse_ok_eq($input, Integer::parse($input), $expected, "Integer::parse");
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
    assert_int_parse!("0", int(Decimal, "0", None));
    assert_int_parse!("1", int(Decimal, "1", None));
    assert_int_parse!("8", int(Decimal, "8", None));
    assert_int_parse!("9", int(Decimal, "9", None));
    assert_int_parse!("10", int(Decimal, "10", None));
    assert_int_parse!("11", int(Decimal, "11", None));
    assert_int_parse!("123456789", int(Decimal, "123456789", None));

    assert_int_parse!("05", int(Decimal, "05", None));
    assert_int_parse!("00005", int(Decimal, "00005", None));
    assert_int_parse!("0123456789", int(Decimal, "0123456789", None));
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
