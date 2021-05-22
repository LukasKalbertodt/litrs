use crate::{Error, parse::{end_dec_digits, first_byte_or_empty}};




#[derive(Debug, Clone, PartialEq)]
pub struct Float<'a> {
    /// Non-empty integer part (before the period).
    integer_part: &'a str,

    /// Optional fractional part. Does not include the period. Is `Some` if a
    /// period exists in the input. Might be `Some("")` for e.g. `3.`.
    fractional_part: Option<&'a str>,

    /// Optional exponent part. Might be empty if there was no exponent part in
    /// the input. Includes the `e` or `E` at the beginning.
    exponent: &'a str,

    /// Optional type suffix.
    type_suffix: Option<FloatType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatType {
    F32,
    F64,
}


impl<'a> Float<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        match first_byte_or_empty(s)? {
            b'0'..=b'9' => Self::parse_impl(s),
            _ => Err(Error::DoesNotStartWithDigit),
        }
    }

    /// Precondition: first byte of string has to be in `b'0'..=b'9'`.
    pub(crate) fn parse_impl(input: &'a str) -> Result<Self, Error> {
        // Integer part.
        let end_integer_part = end_dec_digits(input);
        let (integer_part, rest) = input.split_at(end_integer_part);

        // Fractional part.
        let (fractional_part, rest) = if rest.starts_with('.') {
            let end_fractional_part = end_dec_digits(&rest[1..]);

            let (fractional_part, rest) = rest[1..].split_at(end_fractional_part);
            if fractional_part.starts_with('_') {
                return Err(Error::UnexpectedChar {
                    c: '_',
                    offset: integer_part.len() + 1,
                });
            }

            (Some(fractional_part), rest)
        } else {
            (None, rest)
        };

        // If we have a period that is not followed by decimal digits, the
        // literal must end now.
        if fractional_part == Some("") && !rest.is_empty() {
            return Err(Error::UnexpectedChar {
                c: rest.chars().next().unwrap(),
                offset: integer_part.len() + 1,
            });
        }

        // Optional exponent.
        let (exponent, rest) = if rest.starts_with('e') || rest.starts_with('E') {
            todo!()
        } else {
            ("", rest)
        };

        let type_suffix = match rest {
            "" => None,
            "f32" => Some(FloatType::F32),
            "f64" => Some(FloatType::F64),
            _ => Err(Error::InvalidFloatTypeSuffix { offset: input.len() - rest.len() })?,
        };

        Ok(Self {
            integer_part,
            fractional_part,
            exponent,
            type_suffix,
        })
    }
}
