use std::{fmt, ops::Range};

use crate::{
    Buffer, ParseError,
    err::{perr, ParseErrorKind::*},
    escape::{is_string_continue_skipable_whitespace, unescape},
    parse::first_byte_or_empty,
};


/// A string or raw string literal, e.g. `"foo"`, `"Grüße"` or `r#"a🦊c"d🦀f"#`.
///
/// See [the reference][ref] for more information.
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#string-literals
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLit<B: Buffer> {
    /// The raw input.
    raw: B,

    /// The string value (with all escaped unescaped), or `None` if there were
    /// no escapes. In the latter case, `input` is the string value.
    value: Option<String>,

    /// The number of hash signs in case of a raw string literal, or `None` if
    /// it's not a raw string literal.
    num_hashes: Option<u32>,
}

impl<B: Buffer> StringLit<B> {
    /// Parses the input as a (raw) string literal. Returns an error if the
    /// input is invalid or represents a different kind of literal.
    pub fn parse(input: B) -> Result<Self, ParseError> {
        match first_byte_or_empty(&input)? {
            b'r' | b'"' => Self::parse_impl(input),
            _ => Err(perr(0, InvalidStringLiteralStart)),
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
    pub(crate) fn parse_impl(input: B) -> Result<Self, ParseError> {
        if input.starts_with('r') {
            // Raw string literal
            let num_hashes = input[1..].bytes().position(|b| b != b'#')
                .ok_or(perr(None, InvalidLiteral))?;

            if input.as_bytes().get(1 + num_hashes) != Some(&b'"') {
                return Err(perr(None, InvalidLiteral));
            }
            let start_inner = 1 + num_hashes + 1;
            let hashes = &input[1..num_hashes + 1];

            let mut closing_quote_pos = None;
            for (i, b) in input[start_inner..].bytes().enumerate() {
                if b == b'"' && input[start_inner + i + 1..].starts_with(hashes) {
                    closing_quote_pos = Some(i + start_inner);
                    break;
                }

                if b == b'\r' && input.as_bytes().get(start_inner + i + 1) != Some(&b'\n') {
                    return Err(perr(i + start_inner, IsolatedCr));
                }
            }
            let closing_quote_pos = closing_quote_pos
                .ok_or(perr(None, UnterminatedRawString))?;

            if closing_quote_pos + num_hashes != input.len() - 1 {
                return Err(perr(closing_quote_pos + num_hashes + 1..input.len(), UnexpectedChar));
            }

            Ok(Self {
                raw: input,
                value: None,
                num_hashes: Some(num_hashes as u32),
            })
        } else {
            let mut i = 1;
            let mut end_last_escape = 1;
            let mut value = String::new();
            while i < input.len() - 1 {
                match input.as_bytes()[i] {
                    // Handle "string continue".
                    b'\\' if input.as_bytes()[i + 1] == b'\n' => {
                        value.push_str(&input[end_last_escape..i]);

                        // Find the first non-whitespace character.
                        let end_escape = input[i + 2..].bytes()
                            .position(|b| !is_string_continue_skipable_whitespace(b))
                            .ok_or(perr(None, UnterminatedString))?;

                        i += 2 + end_escape;
                        end_last_escape = i;
                    }
                    b'\\' => {
                        let (c, len) = unescape::<char>(&input[i..input.len() - 1], i)?;
                        value.push_str(&input[end_last_escape..i]);
                        value.push(c);
                        i += len;
                        end_last_escape = i;
                    }
                    b'\r' if input.as_bytes()[i + 1] != b'\n'
                        => return Err(perr(i, IsolatedCr)),
                    b'"' => return Err(perr(i + 1..input.len(), UnexpectedChar)),
                    _ => i += 1,
                }
            }

            if input.as_bytes()[input.len() - 1] != b'"' || input.len() == 1 {
                return Err(perr(None, UnterminatedString));
            }

            // `value` is only empty there was no escape in the input string
            // (with the special case of the input being empty). This means the
            // string value basically equals the input, so we store `None`.
            let value = if value.is_empty() {
                None
            } else {
                // There was an escape in the string, so we need to push the
                // remaining unescaped part of the string still.
                value.push_str(&input[end_last_escape..input.len() - 1]);
                Some(value)
            };
            Ok(Self {
                raw: input,
                value,
                num_hashes: None,
            })
        }
    }
}

impl StringLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn into_owned(self) -> StringLit<String> {
        StringLit {
            raw: self.raw.to_owned(),
            value: self.value,
            num_hashes: self.num_hashes,
        }
    }
}

impl<B: Buffer> fmt::Display for StringLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.raw)
    }
}


#[cfg(test)]
mod tests;
