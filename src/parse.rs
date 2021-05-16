use super::{Bool, Error, Lit};

impl Lit {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let first = s.as_bytes().get(0).ok_or(Error::Empty)?;

        match first {
            b'f' if s == "false" => Ok(Self::Bool(Bool::False)),
            b't' if s == "true" => Ok(Self::Bool(Bool::True)),

            _ => Err(Error::InvalidLiteral),
        }
    }
}
