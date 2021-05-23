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
