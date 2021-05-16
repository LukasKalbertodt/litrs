
mod bool;
mod parse;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_util;

pub use self::{
    bool::Bool,
};


#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Bool(Bool),
    Integer,
    Float,
    Char,
    String,
    Byte,
    ByteString,
}


#[derive(Debug)]
pub enum Error {
    /// The input was an empty string
    Empty,

    /// An unexpected char was encountered.
    UnexpectedChar {
        c: char,
        offset: usize,
    },

    /// Literal was not recognized.
    InvalidLiteral,
}
