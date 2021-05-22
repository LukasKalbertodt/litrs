
mod bool;
mod float;
mod integer;
mod parse;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_util;

pub use self::{
    bool::Bool,
    float::{Float},
    integer::{Integer, IntegerBase, IntegerType},
};


#[derive(Debug, Clone, PartialEq)]
pub enum Lit<'a> {
    Bool(Bool),
    Integer(Integer<'a>),
    Float(Float<'a>),
    Char,
    String,
    Byte,
    ByteString,
}



#[derive(Debug, PartialEq, Eq)]
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

    /// Input does not start with decimal digit when trying to parse an integer.
    DoesNotStartWithDigit,

    /// Integer literal does not contain any valid digits.
    NoValidDigits,

    /// An integer literal overflows the target type.
    IntegerOverflow,

    /// Found a integer type suffix that is invalid.
    InvalidIntegerTypeSuffix {
        offset: usize,
    },

    /// Found a float type suffix that is invalid. Only `f32` and `f64` are
    /// valid.
    InvalidFloatTypeSuffix {
        offset: usize,
    },

    /// Exponent of a float literal does not contain any digits.
    NoExponentDigits,
}
