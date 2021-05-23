use crate::{Buffer, Float};

use super::{Bool, Error, Lit, Integer};


impl<B: Buffer> Lit<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        let first = first_byte_or_empty(&input)?;

        match first {
            b'f' if &*input == "false" => Ok(Self::Bool(Bool::False)),
            b't' if &*input == "true" => Ok(Self::Bool(Bool::True)),

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
                        => Integer::parse_impl(input, digit).map(Lit::Integer),

                    // Potential chars for float literals: `.` as fractional
                    // period, e and E as exponent start and f as type suffix.
                    Some(b'.') | Some(b'e') | Some(b'E') | Some(b'f')
                        => Float::parse_impl(input).map(Lit::Float),

                    _ => Err(Error::UnexpectedChar {
                        c: input[end..].chars().next().unwrap(),
                        offset: end,
                    }),
                }
            },

            _ => Err(Error::InvalidLiteral),
        }
    }
}


pub(crate) fn first_byte_or_empty(s: &str) -> Result<u8, Error> {
    s.as_bytes().get(0).copied().ok_or(Error::Empty)
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
