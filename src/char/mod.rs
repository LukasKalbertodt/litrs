use crate::{Buffer, Error, ErrorKind, escape::unescape, parse::first_byte_or_empty};



#[derive(Debug, Clone, PartialEq)]
pub struct Char<B: Buffer> {
    raw: B,
    value: char,
}

impl<B: Buffer> Char<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        match first_byte_or_empty(&input)? {
            b'\'' => Self::parse_impl(input),
            _ => Err(Error::single(0, ErrorKind::DoesNotStartWithQuote)),
        }
    }

    /// Returns the character value that this literal represents.
    pub fn value(&self) -> char {
        self.value
    }

    /// Precondition: first character in input must be `'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, Error> {
        if input.len() == 1 {
            return Err(Error::spanless(ErrorKind::UnterminatedCharLiteral));
        }
        if *input.as_bytes().last().unwrap() != b'\'' {
            return Err(Error::spanless(ErrorKind::UnterminatedCharLiteral));
        }

        let inner = &input[1..input.len() - 1];
        let first = inner.chars().nth(0).ok_or(Error::spanless(ErrorKind::EmptyCharLiteral))?;
        let (c, len) = match first {
            '\\' => unescape::<char>(inner, 1)?,
            '\'' => return Err(Error::single(1, ErrorKind::UnescapedSingleQuote)),
            other => (other, other.len_utf8()),
        };
        let rest = &inner[len..];

        if !rest.is_empty() {
            return Err(Error::new(len + 1..input.len() - 1, ErrorKind::OverlongCharLiteral));
        }

        Ok(Self {
            raw: input,
            value: c,
        })
    }
}


#[cfg(test)]
mod tests;
