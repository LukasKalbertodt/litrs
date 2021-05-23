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
fn unicode_escapes() {
    check!('\u{0}');
    check!('\u{00}');
    check!('\u{b}');
    check!('\u{B}');
    check!('\u{7e}');
    check!('\u{E4}');
    check!('\u{e4}');
    check!('\u{fc}');
    check!('\u{Fc}');
    check!('\u{fC}');
    check!('\u{FC}');
    check!('\u{b10}');
    check!('\u{B10}');
    check!('\u{0b10}');
    check!('\u{2764}');
    check!('\u{1f602}');
    check!('\u{1F602}');

    check!('\u{0}');
    check!('\u{0__}');
    check!('\u{3_b}');
    check!('\u{1_F_6_0_2}');
    check!('\u{1_F6_02_____}');
}

#[test]
fn invald_ascii_escapes() {
    assert_err!(Char, r"'\x80'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\x81'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\x8a'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\x8F'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xa0'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xB0'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xc3'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xDf'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xff'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xfF'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xFf'", NonAsciiXEscape, 1..5);
    assert_err!(Char, r"'\xFF'", NonAsciiXEscape, 1..5);
}

#[test]
fn invald_escapes() {
    assert_err!(Char, r"'\a'", UnknownEscape, 1..3);
    assert_err!(Char, r"'\y'", UnknownEscape, 1..3);
    assert_err!(Char, r"'\", UnterminatedCharLiteral, None);
    assert_err!(Char, r"'\x'", UnterminatedEscape, 1..3);
    assert_err!(Char, r"'\x1'", UnterminatedEscape, 1..4);
    assert_err!(Char, r"'\xaj'", InvalidXEscape, 1..5);
    assert_err!(Char, r"'\xjb'", InvalidXEscape, 1..5);
}

#[test]
fn invalid_unicode_escapes() {
    assert_err!(Char, r"'\u'", UnicodeEscapeWithoutBrace, 1..3);
    assert_err!(Char, r"'\u '", UnicodeEscapeWithoutBrace, 1..3);
    assert_err!(Char, r"'\u3'", UnicodeEscapeWithoutBrace, 1..3);

    assert_err!(Char, r"'\u{'", UnterminatedUnicodeEscape, 1..4);
    assert_err!(Char, r"'\u{12'", UnterminatedUnicodeEscape, 1..6);
    assert_err!(Char, r"'\u{a0b'", UnterminatedUnicodeEscape, 1..7);
    assert_err!(Char, r"'\u{a0_b  '", UnterminatedUnicodeEscape, 1..10);

    assert_err!(Char, r"'\u{_}'", InvalidStartOfUnicodeEscape, 4);
    assert_err!(Char, r"'\u{_5f}'", InvalidStartOfUnicodeEscape, 4);

    assert_err!(Char, r"'\u{x}'", NonHexDigitInUnicodeEscape, 4);
    assert_err!(Char, r"'\u{0x}'", NonHexDigitInUnicodeEscape, 5);
    assert_err!(Char, r"'\u{3bx}'", NonHexDigitInUnicodeEscape, 6);
    assert_err!(Char, r"'\u{3b_x}'", NonHexDigitInUnicodeEscape, 7);
    assert_err!(Char, r"'\u{4x_}'", NonHexDigitInUnicodeEscape, 5);

    assert_err!(Char, r"'\u{1234567}'", TooManyDigitInUnicodeEscape, 10);
    assert_err!(Char, r"'\u{1234567}'", TooManyDigitInUnicodeEscape, 10);
    assert_err!(Char, r"'\u{1_23_4_56_7}'", TooManyDigitInUnicodeEscape, 14);
    assert_err!(Char, r"'\u{abcdef123}'", TooManyDigitInUnicodeEscape, 10);

    assert_err!(Char, r"'\u{110000}'", InvalidUnicodeEscapeChar, 1..10);
}

#[test]
fn parse_err() {
    assert_err!(Char, r"''", EmptyCharLiteral, None);
    assert_err!(Char, r"' ''", OverlongCharLiteral, 2..3);

    assert_err!(Char, r"'", UnterminatedCharLiteral, None);
    assert_err!(Char, r"'a", UnterminatedCharLiteral, None);
    assert_err!(Char, r"'\n", UnterminatedCharLiteral, None);
    assert_err!(Char, r"'\x35", UnterminatedCharLiteral, None);

    assert_err!(Char, r"'ab'", OverlongCharLiteral, 2..3);
    assert_err!(Char, r"'a _'", OverlongCharLiteral, 2..4);
    assert_err!(Char, r"'\n3'", OverlongCharLiteral, 3..4);

    assert_err!(Char, r"", Empty, None);

    assert_err!(Char, r"'''", UnescapedSingleQuote, 1);
    assert_err!(Char, r"''''", UnescapedSingleQuote, 1);
}
