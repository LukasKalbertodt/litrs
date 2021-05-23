use crate::{Error, ErrorKind, parse::hex_digit_value};


/// Must start with `\`
pub(crate) fn unescape<E: Escapee>(input: &str, offset: usize) -> Result<(E, usize), Error> {
    let first = input.as_bytes().get(1)
        .ok_or(Error::single(offset, ErrorKind::UnterminatedEscape))?;
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
                .ok_or(Error::new(offset..offset + input.len(), ErrorKind::UnterminatedEscape))?
                .as_bytes();
            let first = hex_digit_value(hex_string[0])
                .ok_or(Error::new(offset..offset + 4, ErrorKind::InvalidXEscape))?;
            let second = hex_digit_value(hex_string[1])
                .ok_or(Error::new(offset..offset + 4, ErrorKind::InvalidXEscape))?;
            let value = second + 16 * first;

            if E::SUPPORTS_UNICODE && value > 0x7F {
                return Err(Error::new(offset..offset + 4, ErrorKind::NonAsciiXEscape));
            }

            (E::from_byte(value), 4)
        },

        // Unicode escape
        b'u' => {
            if !E::SUPPORTS_UNICODE {
                return Err(Error::new(offset..offset + 2, ErrorKind::UnicodeEscapeInByteLiteral));
            }

            if input.as_bytes().get(2) != Some(&b'{') {
                return Err(Error::new(offset..offset + 2, ErrorKind::UnicodeEscapeWithoutBrace));
            }

            let closing_pos = input.bytes().position(|b| b == b'}').ok_or(Error::new(
                offset..offset + input.len(),
                ErrorKind::UnterminatedUnicodeEscape,
            ))?;

            let inner = &input[3..closing_pos];
            if inner.as_bytes().first() == Some(&b'_') {
                return Err(Error::single(4, ErrorKind::InvalidStartOfUnicodeEscape));
            }

            let mut v: u32 = 0;
            let mut digit_count = 0;
            for (i, b) in inner.bytes().enumerate() {
                if b == b'_'{
                    continue;
                }

                let digit = hex_digit_value(b)
                    .ok_or(Error::single(offset + 3 + i, ErrorKind::NonHexDigitInUnicodeEscape))?;

                if digit_count == 6 {
                    return Err(Error::single(
                        offset + 3 + i,
                        ErrorKind::TooManyDigitInUnicodeEscape,
                    ));
                }
                digit_count += 1;
                v = 16 * v + digit as u32;
            }

            let c = char::from_u32(v)
                .ok_or(Error::new(offset..closing_pos + 1, ErrorKind::InvalidUnicodeEscapeChar))?;

            (E::from_char(c), closing_pos + 1)
        }

        _ => return Err(Error::new(offset..offset + 2, ErrorKind::UnknownEscape)),
    };

    Ok(out)
}

pub(crate) trait Escapee {
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
