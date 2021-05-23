use crate::{Lit, test_util::assert_parse_ok_eq};
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
    assert_err!(Char::parse(r"'\x80'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\x81'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\x8a'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\x8F'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xa0'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xB0'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xc3'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xDf'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xff'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xfF'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xFf'"), NonAsciiXEscape, 1..5);
    assert_err!(Char::parse(r"'\xFF'"), NonAsciiXEscape, 1..5);
}

#[test]
fn parse_err() {
    assert_err!(Char::parse(r"''"), EmptyCharLiteral, None);

    assert_err!(Char::parse(r"'"), UnterminatedCharLiteral, None);
    assert_err!(Char::parse(r"'a"), UnterminatedCharLiteral, None);
    assert_err!(Char::parse(r"'\n"), UnterminatedCharLiteral, None);
    assert_err!(Char::parse(r"'\x35"), UnterminatedCharLiteral, None);

    assert_err!(Char::parse(r"'ab'"), OverlongCharLiteral, 2..4);
    assert_err!(Char::parse(r"'a _'"), OverlongCharLiteral, 2..5);

    assert_err!(Char::parse(r""), Empty, None);

    assert_err!(Char::parse(r"'''"), UnescapedSingleQuote, 1);
}
