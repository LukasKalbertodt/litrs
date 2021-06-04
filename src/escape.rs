use crate::{ParseError, err::{perr, ParseErrorKind::*}, parse::hex_digit_value};


/// Must start with `\`
pub(crate) fn unescape<E: Escapee>(input: &str, offset: usize) -> Result<(E, usize), ParseError> {
    let first = input.as_bytes().get(1)
        .ok_or(perr(offset, UnterminatedEscape))?;
    let out = match first {
        // Quote escapes
        b'\'' => (E::from_byte(b'\''), 2),
        b'"' => (E::from_byte(b'"'), 2),

        // Ascii escapes
        b'n' => (E::from_byte(b'\n'), 2),
        b'r' => (E::from_byte(b'\r'), 2),
        b't' => (E::from_byte(b'\t'), 2),
        b'\\' => (E::from_byte(b'\\'), 2),
        b'0' => (E::from_byte(b'\0'), 2),
        b'x' => {
            let hex_string = input.get(2..4)
                .ok_or(perr(offset..offset + input.len(), UnterminatedEscape))?
                .as_bytes();
            let first = hex_digit_value(hex_string[0])
                .ok_or(perr(offset..offset + 4, InvalidXEscape))?;
            let second = hex_digit_value(hex_string[1])
                .ok_or(perr(offset..offset + 4, InvalidXEscape))?;
            let value = second + 16 * first;

            if E::SUPPORTS_UNICODE && value > 0x7F {
                return Err(perr(offset..offset + 4, NonAsciiXEscape));
            }

            (E::from_byte(value), 4)
        },

        // Unicode escape
        b'u' => {
            if !E::SUPPORTS_UNICODE {
                return Err(perr(offset..offset + 2, UnicodeEscapeInByteLiteral));
            }

            if input.as_bytes().get(2) != Some(&b'{') {
                return Err(perr(offset..offset + 2, UnicodeEscapeWithoutBrace));
            }

            let closing_pos = input.bytes().position(|b| b == b'}')
                .ok_or(perr(offset..offset + input.len(), UnterminatedUnicodeEscape))?;

            let inner = &input[3..closing_pos];
            if inner.as_bytes().first() == Some(&b'_') {
                return Err(perr(4, InvalidStartOfUnicodeEscape));
            }

            let mut v: u32 = 0;
            let mut digit_count = 0;
            for (i, b) in inner.bytes().enumerate() {
                if b == b'_'{
                    continue;
                }

                let digit = hex_digit_value(b)
                    .ok_or(perr(offset + 3 + i, NonHexDigitInUnicodeEscape))?;

                if digit_count == 6 {
                    return Err(perr(offset + 3 + i, TooManyDigitInUnicodeEscape));
                }
                digit_count += 1;
                v = 16 * v + digit as u32;
            }

            let c = char::from_u32(v)
                .ok_or(perr(offset..closing_pos + 1, InvalidUnicodeEscapeChar))?;

            (E::from_char(c), closing_pos + 1)
        }

        _ => return Err(perr(offset..offset + 2, UnknownEscape)),
    };

    Ok(out)
}

pub(crate) trait Escapee: Into<char> {
    const SUPPORTS_UNICODE: bool;
    fn from_byte(b: u8) -> Self;
    fn from_char(c: char) -> Self;
}

impl Escapee for u8 {
    const SUPPORTS_UNICODE: bool = false;
    fn from_byte(b: u8) -> Self {
        b
    }
    fn from_char(_: char) -> Self {
        panic!("bug: `<u8 as Escapee>::from_char` was called");
    }
}

impl Escapee for char {
    const SUPPORTS_UNICODE: bool = true;
    fn from_byte(b: u8) -> Self {
        b.into()
    }
    fn from_char(c: char) -> Self {
        c
    }
}

/// Checks whether the character is skipped after a string continue start
/// (unescaped backlash followed by `\n`).
pub(crate) fn is_string_continue_skipable_whitespace(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r'
}

/// Unescapes a whole string or byte string.
pub(crate) fn unescape_string<E: Escapee>(
    input: &str,
    offset: usize,
) -> Result<Option<String>, ParseError> {
    let mut i = offset;
    let mut end_last_escape = offset;
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
                let (c, len) = unescape::<E>(&input[i..input.len() - 1], i)?;
                value.push_str(&input[end_last_escape..i]);
                value.push(c.into());
                i += len;
                end_last_escape = i;
            }
            b'\r' if input.as_bytes()[i + 1] != b'\n'
                => return Err(perr(i, IsolatedCr)),
            b'"' => return Err(perr(i + 1..input.len(), UnexpectedChar)),
            b if !E::SUPPORTS_UNICODE && !b.is_ascii()
                => return Err(perr(i, NonAsciiInByteLiteral)),
            _ => i += 1,
        }
    }

    if input.as_bytes()[input.len() - 1] != b'"' || input.len() == offset {
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

    Ok(value)
}
