use crate::{
    Lit, Error,
    test_util::assert_parse_ok_eq,
};
use super::Char;

// ===== Utility functions =======================================================================

macro_rules! check {
    ($lit:literal) => {
        let input = stringify!($lit);
        let expected = Char {
            raw: input,
            value: $lit,
        };

        assert_parse_ok_eq(input, Char::parse(input), expected.clone(), "Char::parse");
        assert_parse_ok_eq(input, Lit::parse(input), Lit::Char(expected), "Lit::parse");
        assert_eq!(Char::parse(input).unwrap().value(), $lit);
    };
}


// ===== Actual tests ============================================================================


#[test]
fn alphanumeric() {
    check!('a');
    check!('b');
    check!('y');
    check!('z');
    check!('A');
    check!('B');
    check!('Y');
    check!('Z');

    check!('0');
    check!('1');
    check!('8');
    check!('9');
}

#[test]
fn special_chars() {
    check!(' ');
    check!('!');
    check!('"');
    check!('#');
    check!('$');
    check!('%');
    check!('&');
    check!('(');
    check!(')');
    check!('*');
    check!('+');
    check!(',');
    check!('-');
    check!('.');
    check!('/');
    check!(':');
    check!(';');
    check!('<');
    check!('=');
    check!('>');
    check!('?');
    check!('@');
    check!('[');
    check!(']');
    check!('^');
    check!('_');
    check!('`');
    check!('{');
    check!('|');
    check!('}');
    check!('~');
}

#[test]
fn quote_escapes() {
    check!('\'');
    check!('\"');
}

#[test]
fn ascii_escapes() {
    check!('\n');
    check!('\r');
    check!('\t');
    check!('\\');
    check!('\0');

    check!('\x00');
    check!('\x01');
    check!('\x0c');
    check!('\x0D');
    check!('\x13');
    check!('\x30');
    check!('\x30');
    check!('\x4B');
    check!('\x6b');
    check!('\x7F');
    check!('\x7f');
}

#[test]
fn invald_ascii_escapes() {
    assert_eq!(Char::parse(r"'\x80'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\x81'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\x8a'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\x8F'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xa0'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xB0'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xc3'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xDf'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xff'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xfF'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xFf'"), Err(Error::NonAsciiXEscape));
    assert_eq!(Char::parse(r"'\xFF'"), Err(Error::NonAsciiXEscape));
}

#[test]
fn parse_err() {
    assert_eq!(Char::parse(r"''"), Err(Error::EmptyCharLiteral));

    assert_eq!(Char::parse(r"'"), Err(Error::UnterminatedLiteral));
    assert_eq!(Char::parse(r"'a"), Err(Error::UnterminatedLiteral));
    assert_eq!(Char::parse(r"'\n"), Err(Error::UnterminatedLiteral));
    assert_eq!(Char::parse(r"'\x35"), Err(Error::UnterminatedLiteral));

    assert_eq!(Char::parse(r"'ab'"), Err(Error::OverlongCharLiteral));
    assert_eq!(Char::parse(r"'a _'"), Err(Error::OverlongCharLiteral));

    assert_eq!(Char::parse(r""), Err(Error::Empty));

    assert!(Char::parse(r"'''").is_err());
}
