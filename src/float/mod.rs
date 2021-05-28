use std::fmt;

use crate::{
    Buffer, ParseError,
    err::{perr, ParseErrorKind::*},
    parse::{end_dec_digits, first_byte_or_empty},
};



/// A floating point literal, e.g. `3.14`, `8.`, `135e12`, `27f32` or `1.956e2f64`.
///
/// This kind of literal has several forms, but generally consists of a main
/// number part, an optional exponent and an optional type suffix. See
/// [the reference][ref] for more information.
///
/// A leading minus sign `-` is not part of the literal grammar! `-3.14` are two
/// tokens in the Rust grammar.
///
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#floating-point-literals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatLit<B: Buffer> {
    /// Basically the whole literal, but without the type suffix. Other `usize`
    /// fields in this struct partition this string. `end_integer_part` is
    /// always <= `end_fractional_part`.
    ///
    /// ```text
    ///    12_3.4_56e789
    ///        ╷    ╷
    ///        |    └ end_fractional_part = 9
    ///        └ end_integer_part = 4
    ///
    ///    246.
    ///       ╷╷
    ///       |└ end_fractional_part = 4
    ///       └ end_integer_part = 3
    ///
    ///    1234e89
    ///        ╷
    ///        |
    ///        └ end_integer_part = end_fractional_part = 4
    /// ```
    number_part: B,

    /// The first index not part of the integer part anymore. Since the integer
    /// part is at the start, this is also the length of that part.
    end_integer_part: usize,

    /// The first index after the fractional part.
    end_fractional_part: usize,

    /// Optional type suffix.
    type_suffix: Option<FloatType>,
}

/// All possible float type suffixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
}

impl<B: Buffer> FloatLit<B> {
    /// Parses the input as a floating point literal. Returns an error if the
    /// input is invalid or represents a different kind of literal.
    pub fn parse(s: B) -> Result<Self, ParseError> {
        match first_byte_or_empty(&s)? {
            b'0'..=b'9' => Self::parse_impl(s),
            _ => Err(perr(0, DoesNotStartWithDigit)),
        }
    }

    /// Returns the whole number part (including integer part, fractional part
    /// and exponent), but without the type suffix. If you want an actual
    /// floating point value, you need to parse this string, e.g. with
    /// `f32::from_str` or an external crate.
    pub fn number_part(&self) -> &str {
        &self.number_part
    }

    /// Returns the non-empty integer part of this literal.
    pub fn integer_part(&self) -> &str {
        &(*self.number_part)[..self.end_integer_part]
    }

    /// Returns the optional fractional part of this literal. Does not include
    /// the period. If a period exists in the input, `Some` is returned, `None`
    /// otherwise. Note that `Some("")` might be returned, e.g. for `3.`.
    pub fn fractional_part(&self) -> Option<&str> {
        if self.end_integer_part == self.end_fractional_part {
            None
        } else {
            Some(&(*self.number_part)[self.end_integer_part + 1..self.end_fractional_part])
        }
    }

    /// Optional exponent part. Might be empty if there was no exponent part in
    /// the input. Includes the `e` or `E` at the beginning.
    pub fn exponent_part(&self) -> &str {
        &(*self.number_part)[self.end_fractional_part..]
    }

    /// The optional type suffix.
    pub fn type_suffix(&self) -> Option<FloatType> {
        self.type_suffix
    }

    /// Precondition: first byte of string has to be in `b'0'..=b'9'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, ParseError> {
        // Integer part.
        let end_integer_part = end_dec_digits(&input);
        let rest = &input[end_integer_part..];


        // Fractional part.
        let end_fractional_part = if rest.as_bytes().get(0) == Some(&b'.') {
            // The fractional part must not start with `_`.
            if rest.as_bytes().get(1) == Some(&b'_') {
                return Err(perr(end_integer_part + 1, UnexpectedChar));
            }

            end_dec_digits(&rest[1..]) + 1 + end_integer_part
        } else {
            end_integer_part
        };
        let rest = &input[end_fractional_part..];

        // If we have a period that is not followed by decimal digits, the
        // literal must end now.
        if end_integer_part + 1 == end_fractional_part && !rest.is_empty() {
            return Err(perr(end_integer_part + 1, UnexpectedChar));
        }


        // Optional exponent.
        let end_number_part = if rest.starts_with('e') || rest.starts_with('E') {
            // Strip single - or + sign at the beginning.
            let exp_number_start = match rest.as_bytes().get(1) {
                Some(b'-') | Some(b'+') => 2,
                _ => 1,
            };

            // Find end of exponent and make sure there is at least one digit.
            let end_exponent = end_dec_digits(&rest[exp_number_start..]) + exp_number_start;
            if !rest[exp_number_start..end_exponent].bytes().any(|b| matches!(b, b'0'..=b'9')) {
                return Err(perr(
                    end_fractional_part..end_fractional_part + end_exponent,
                    NoExponentDigits,
                ));
            }

            end_exponent + end_fractional_part
        } else {
            end_fractional_part
        };


        // Type suffix
        let type_suffix = match &input[end_number_part..] {
            "" => None,
            "f32" => Some(FloatType::F32),
            "f64" => Some(FloatType::F64),
            _ => return Err(perr(end_number_part..input.len(), InvalidFloatTypeSuffix)),
        };

        Ok(Self {
            number_part: input.cut(0..end_number_part),
            end_integer_part,
            end_fractional_part,
            type_suffix,
        })
    }
}

impl FloatLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn to_owned(&self) -> FloatLit<String> {
        FloatLit {
            number_part: self.number_part.to_owned(),
            end_integer_part: self.end_integer_part,
            end_fractional_part: self.end_fractional_part,
            type_suffix: self.type_suffix,
        }
    }
}

impl<B: Buffer> fmt::Display for FloatLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self.type_suffix {
            None => "",
            Some(FloatType::F32) => "f32",
            Some(FloatType::F64) => "f64",
        };
        write!(f, "{}{}", self.number_part(), suffix)
    }
}


#[cfg(test)]
mod tests;
