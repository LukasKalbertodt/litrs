use crate::Error;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bool {
    False,
    True,
}

impl Bool {
    pub fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "false" => Ok(Self::False),
            "true" => Ok(Self::True),
            _ => Err(Error::InvalidLiteral)
        }
    }
}


#[cfg(test)]
mod tests;
