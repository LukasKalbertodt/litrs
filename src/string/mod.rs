use std::ops::Range;

use crate::{Buffer, Error, ErrorKind, parse::first_byte_or_empty};


#[derive(Debug, Clone, PartialEq)]
pub struct StringLit<B: Buffer> {
    /// The raw input.
    raw: B,

    /// The string value (with all escaped unescaped), or `None` if there were
    /// no escapes. In the latter case, `input` is the string value.
    value: Option<std::string::String>,

    /// The number of hash signs in case of a raw string literal, or `None` if
    /// it's not a raw string literal.
    num_hashes: Option<u32>,
}

impl<B: Buffer> StringLit<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        match first_byte_or_empty(&input)? {
            b'r' | b'"' => Self::parse_impl(input),
            _ => Err(Error::single(0, ErrorKind::InvalidStringLiteralStart)),
        }
    }

    /// Returns the string value this literal represents (where all escapes have
    /// been turned into their respective values).
    pub fn value(&self) -> &str {
        self.value.as_deref().unwrap_or(&self.raw[self.inner_range()])
    }

    /// Like `value` but returns a potentially owned version of the value.
    ///
    /// The return value is either `Cow<'static, str>` if `B = String`, or
    /// `Cow<'a, str>` if `B = &'a str`.
    pub fn into_value(self) -> B::Cow {
        let inner_range = self.inner_range();
        let Self { raw, value, .. } = self;
        value.map(B::Cow::from).unwrap_or_else(|| raw.cut(inner_range).into_cow())
    }

    /// Returns whether this literal is a raw string literal (starting with
    /// `r`).
    pub fn is_raw_string(&self) -> bool {
        self.num_hashes.is_some()
    }

    /// The range within `self.raw` that excludes the quotes and potential `r#`.
    fn inner_range(&self) -> Range<usize> {
        match self.num_hashes {
            None => 1..self.raw.len() - 1,
            Some(n) => 1 + n as usize + 1..self.raw.len() - n as usize - 1,
        }
    }

    /// Precondition: input has to start with either `"` or `r`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, Error> {
        if input.starts_with('r') {
            // Raw string literal
            let num_hashes = input[1..].bytes().position(|b| b != b'#')
                .ok_or(Error::spanless(ErrorKind::InvalidLiteral))?;

            if input.as_bytes().get(1 + num_hashes) != Some(&b'"') {
                return Err(Error::spanless(ErrorKind::InvalidLiteral));
            }
            let start_inner = 1 + num_hashes + 1;
            let hashes = &input[1..num_hashes + 1];

            // Find the end of the string and make sure there is nothing afterwards.
            let closing_quote_pos = input[start_inner..].bytes()
                .enumerate()
                .position(|(i, b)| b == b'"' && input[start_inner + i + 1..].starts_with(hashes))
                .map(|pos| pos + start_inner)
                .ok_or(Error::spanless(ErrorKind::UnterminatedRawString))?;

            if closing_quote_pos + num_hashes != input.len() - 1 {
                return Err(Error::new(
                    closing_quote_pos + num_hashes..input.len(),
                    ErrorKind::UnexpectedChar,
                ));
            }

            Ok(Self {
                raw: input,
                value: None,
                num_hashes: Some(num_hashes as u32),
            })
        } else {
            todo!()
        }
    }
}


#[cfg(test)]
mod tests;
