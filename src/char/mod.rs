use crate::{Buffer, Error, escape::unescape, parse::first_byte_or_empty};



#[derive(Debug, Clone, PartialEq)]
pub struct Char<B: Buffer> {
    raw: B,
    value: char,
}

impl<B: Buffer> Char<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        match first_byte_or_empty(&input)? {
            b'\'' => Self::parse_impl(input),
            _ => Err(Error::DoesNotStartWithDigit),
        }
    }

    /// Returns the character value that this literal represents.
    pub fn value(&self) -> char {
        self.value
    }

    /// Precondition: first character in input must be `'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, Error> {
        let inner = &(*input)[1..];
        let (c, len) = match inner.chars().nth(0).ok_or(Error::UnterminatedLiteral)? {
            '\\' => unescape::<char>(inner)?,
            '\'' if input.len() == 2 => return Err(Error::EmptyCharLiteral),
            '\'' => return Err(Error::UnescapedQuote),
            other => (other, other.len_utf8()),
        };
        let rest = &inner[len..];

        if rest.len() > 1 {
            return Err(Error::OverlongCharLiteral);
        } else if rest != "'" {
            return Err(Error::UnterminatedLiteral);
        }

        Ok(Self {
            raw: input,
            value: c,
        })
    }
}


#[cfg(test)]
mod tests;
