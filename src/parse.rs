use crate::{
    BoolLit,
    Buffer,
    ByteLit,
    ByteStringLit,
    CharLit,
    ParseError,
    FloatLit,
    IntegerLit,
    Literal,
    StringLit,
    err::{perr, ParseErrorKind::*},
};


impl<B: Buffer> Literal<B> {
    /// Parses the given input as a Rust literal.
    pub fn parse(input: B) -> Result<Self, ParseError> {
        let first = first_byte_or_empty(&input)?;
        let second = input.as_bytes().get(1).copied();

        match first {
            b'f' if &*input == "false" => Ok(Self::Bool(BoolLit::False)),
            b't' if &*input == "true" => Ok(Self::Bool(BoolLit::True)),

            // A number literal (integer or float).
            digit @ b'0'..=b'9' => {
                // To figure out whether this is a float or integer, we do some
                // quick inspection here. Yes, this is technically duplicate
                // work with what is happening in the integer/float parse
                // methods, but it makes the code way easier for now and won't
                // be a huge performance loss.
                let end = 1 + end_dec_digits(&input[1..]);
                match input.as_bytes().get(end) {
                    // Potential chars in integer literals: b, o, x for base; u
                    // and i for type suffix.
                    None | Some(b'b') | Some(b'o') | Some(b'x') | Some(b'u') | Some(b'i')
                        => IntegerLit::parse_impl(input, digit).map(Literal::Integer),

                    // Potential chars for float literals: `.` as fractional
                    // period, e and E as exponent start and f as type suffix.
                    Some(b'.') | Some(b'e') | Some(b'E') | Some(b'f')
                        => FloatLit::parse_impl(input).map(Literal::Float),

                    _ => Err(perr(end, UnexpectedChar)),
                }
            },

            b'\'' => CharLit::parse_impl(input).map(Literal::Char),
            b'"' | b'r' => StringLit::parse_impl(input).map(Literal::String),

            b'b' if second == Some(b'\'') => ByteLit::parse_impl(input).map(Literal::Byte),
            b'b' if second == Some(b'r') || second == Some(b'"')
                => ByteStringLit::parse_impl(input).map(Literal::ByteString),

            _ => Err(perr(None, InvalidLiteral)),
        }
    }
}


pub(crate) fn first_byte_or_empty(s: &str) -> Result<u8, ParseError> {
    s.as_bytes().get(0).copied().ok_or(perr(None, Empty))
}

/// Returns the index of the first non-underscore, non-decimal digit in `input`,
/// or the `input.len()` if all characters are decimal digits.
pub(crate) fn end_dec_digits(input: &str) -> usize {
    input.bytes()
        .position(|b| !matches!(b, b'_' | b'0'..=b'9'))
        .unwrap_or(input.len())
}

pub(crate) fn hex_digit_value(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        _ => None,
    }
}
