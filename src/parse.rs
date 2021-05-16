use super::{Bool, Error, Lit};

impl Lit {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let first = first_byte_or_empty(s)?;

        match first {
            b'f' if s == "false" => Ok(Self::Bool(Bool::False)),
            b't' if s == "true" => Ok(Self::Bool(Bool::True)),

            _ => Err(Error::InvalidLiteral),
        }
    }
}


pub(crate) fn first_byte_or_empty(s: &str) -> Result<u8, Error> {
    s.as_bytes().get(0).copied().ok_or(Error::Empty)
}
