use std::fmt;

use crate::{
    Buffer, ParseError,
    err::{perr, ParseErrorKind::*},
    escape::unescape,
    parse::first_byte_or_empty,
};


/// A character literal, e.g. `'g'` or `'🦊'`.
///
/// See [the reference][ref] for more information.
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#character-literals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharLit<B: Buffer> {
    raw: B,
    value: char,
}

impl<B: Buffer> CharLit<B> {
    /// Parses the input as a character literal. Returns an error if the input
    /// is invalid or represents a different kind of literal.
    pub fn parse(input: B) -> Result<Self, ParseError> {
        match first_byte_or_empty(&input)? {
            b'\'' => Self::parse_impl(input),
            _ => Err(perr(0, DoesNotStartWithQuote)),
        }
    }

    /// Returns the character value that this literal represents.
    pub fn value(&self) -> char {
        self.value
    }

    /// Precondition: first character in input must be `'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, ParseError> {
        if input.len() == 1 {
            return Err(perr(None, UnterminatedCharLiteral));
        }
        if *input.as_bytes().last().unwrap() != b'\'' {
            return Err(perr(None, UnterminatedCharLiteral));
        }

        let inner = &input[1..input.len() - 1];
        let first = inner.chars().nth(0).ok_or(perr(None, EmptyCharLiteral))?;
        let (c, len) = match first {
            '\'' => return Err(perr(1, UnescapedSingleQuote)),
            '\n' | '\t' | '\r'
                => return Err(perr(1, UnescapedSpecialWhitespace)),

            '\\' => unescape::<char>(inner, 1)?,
            other => (other, other.len_utf8()),
        };
        let rest = &inner[len..];

        if !rest.is_empty() {
            return Err(perr(len + 1..input.len() - 1, OverlongCharLiteral));
        }

        Ok(Self {
            raw: input,
            value: c,
        })
    }
}

impl CharLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn to_owned(&self) -> CharLit<String> {
        CharLit {
            raw: self.raw.to_owned(),
            value: self.value,
        }
    }
}

impl<B: Buffer> fmt::Display for CharLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.raw)
    }
}


#[cfg(test)]
mod tests;
