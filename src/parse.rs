use super::{Bool, Error, Lit, Integer};


impl<'a> Lit<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        let first = first_byte_or_empty(s)?;

        match first {
            b'f' if s == "false" => Ok(Self::Bool(Bool::False)),
            b't' if s == "true" => Ok(Self::Bool(Bool::True)),

            digit @ b'0'..=b'9' => Integer::parse_impl(s, digit).map(Lit::Integer),

            _ => Err(Error::InvalidLiteral),
        }
    }
}


pub(crate) fn first_byte_or_empty(s: &str) -> Result<u8, Error> {
    s.as_bytes().get(0).copied().ok_or(Error::Empty)
}
