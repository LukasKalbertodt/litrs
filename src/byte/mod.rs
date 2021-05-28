use core::fmt;

use crate::{
    Buffer, ParseError,
    err::{perr, ParseErrorKind::*},
    escape::unescape,
};


/// A (single) byte literal, e.g. `b'k'` or `b'!'`.
///
/// See [the reference][ref] for more information.
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#byte-literals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteLit<B: Buffer> {
    raw: B,
    value: u8,
}

impl<B: Buffer> ByteLit<B> {
    /// Parses the input as a byte literal. Returns an error if the input is
    /// invalid or represents a different kind of literal.
    pub fn parse(input: B) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(perr(None, Empty));
        }
        if !input.starts_with("b'") {
            return Err(perr(None, InvalidByteLiteralStart));
        }

        Self::parse_impl(input)
    }

    /// Returns the byte value that this literal represents.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Precondition: must start with `b'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, ParseError> {
        if input.len() == 2 {
            return Err(perr(None, UnterminatedByteLiteral));
        }
        if *input.as_bytes().last().unwrap() != b'\'' {
            return Err(perr(None, UnterminatedByteLiteral));
        }

        let inner = &input[2..input.len() - 1];
        let first = inner.as_bytes().get(0).ok_or(perr(None, EmptyByteLiteral))?;
        let (c, len) = match first {
            b'\'' => return Err(perr(2, UnescapedSingleQuote)),
            b'\n' | b'\t' | b'\r'
                => return Err(perr(2, UnescapedSpecialWhitespace)),

            b'\\' => unescape::<u8>(inner, 2)?,
            other if other.is_ascii() => (*other, 1),
            _ => return Err(perr(2, NonAsciiInByteLiteral)),
        };
        let rest = &inner[len..];

        if !rest.is_empty() {
            return Err(perr(len + 2..input.len() - 1, OverlongByteLiteral));
        }

        Ok(Self {
            raw: input,
            value: c,
        })
    }
}

impl ByteLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn to_owned(&self) -> ByteLit<String> {
        ByteLit {
            raw: self.raw.to_owned(),
            value: self.value,
        }
    }
}

impl<B: Buffer> fmt::Display for ByteLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.raw)
    }
}

#[cfg(test)]
mod tests;
