use crate::{Buffer, Error, ErrorKind, escape::unescape};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteLit<B: Buffer> {
    raw: B,
    value: u8,
}

impl<B: Buffer> ByteLit<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(Error::spanless(ErrorKind::Empty));
        }
        if !input.starts_with("b'") {
            return Err(Error::spanless(ErrorKind::InvalidByteLiteralStart));
        }

        Self::parse_impl(input)
    }

    /// Returns the byte value that this literal represents.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Precondition: must start with `b'`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, Error> {
        if input.len() == 2 {
            return Err(Error::spanless(ErrorKind::UnterminatedByteLiteral));
        }
        if *input.as_bytes().last().unwrap() != b'\'' {
            return Err(Error::spanless(ErrorKind::UnterminatedByteLiteral));
        }

        let inner = &input[2..input.len() - 1];
        let first = inner.as_bytes().get(0).ok_or(Error::spanless(ErrorKind::EmptyByteLiteral))?;
        let (c, len) = match first {
            b'\'' => return Err(Error::single(2, ErrorKind::UnescapedSingleQuote)),
            b'\n' | b'\t' | b'\r'
                => return Err(Error::single(2, ErrorKind::UnescapedSpecialWhitespace)),

            b'\\' => unescape::<u8>(inner, 2)?,
            other if other.is_ascii() => (*other, 1),
            _ => return Err(Error::single(2, ErrorKind::NonAsciiInByteLiteral)),
        };
        let rest = &inner[len..];

        if !rest.is_empty() {
            return Err(Error::new(len + 2..input.len() - 1, ErrorKind::OverlongByteLiteral));
        }

        Ok(Self {
            raw: input,
            value: c,
        })
    }
}


#[cfg(test)]
mod tests;
