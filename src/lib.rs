
mod bool;
mod float;
mod integer;
mod parse;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_util;

use std::ops::{Deref, Range};

pub use self::{
    bool::Bool,
    float::{Float, FloatType},
    integer::{Integer, IntegerBase, IntegerType},
};


pub type OwnedLit = Lit<String>;
pub type SharedLit<'a> = Lit<&'a str>;


#[derive(Debug, Clone, PartialEq)]
pub enum Lit<B: Buffer> {
    Bool(Bool),
    Integer(Integer<B>),
    Float(Float<B>),
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

/// A shared or owned string buffer, implemented for `String` and `&str`.
///
/// This is trait is implementation detail of this library, cannot be
/// implemented in other crates and is not subject to semantic versioning.
/// `litrs` only gurantees that this trait is implemented for `String` and
/// `&str`.
pub trait Buffer: sealed::Sealed + Deref<Target = str> {
    /// Cuts away some characters at the beginning and some at the end. Given
    /// range has to be in bounds.
    #[doc(hidden)]
    fn cut(self, range: Range<usize>) -> Self;
}

mod sealed {
    pub trait Sealed {}
}

impl<'a> sealed::Sealed for &'a str {}
impl<'a> Buffer for &'a str {
    fn cut(self, range: Range<usize>) -> Self {
        &self[range]
    }
}

impl sealed::Sealed for String {}
impl Buffer for String {
    fn cut(mut self, range: Range<usize>) -> Self {
        // This is not the most efficient way, but it works. First we cut the
        // end, then the beginning. Note that `drain` also removes the range if
        // the iterator is not consumed.
        self.truncate(range.end);
        self.drain(..range.start);
        self
    }
}
