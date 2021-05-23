#[cfg(test)]
#[macro_use]
mod test_util;

#[cfg(test)]
mod tests;

mod bool;
mod char;
mod escape;
mod float;
mod integer;
mod parse;


use std::ops::{Deref, Range};

pub use self::{
    bool::Bool,
    char::Char,
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
    Char(Char<B>),
    String,
    Byte,
    ByteString,
}


#[derive(Debug, Clone)]
pub struct Error {
    span: Option<Range<usize>>,
    kind: ErrorKind,
}

impl Error {
    fn new(span: Range<usize>, kind: ErrorKind) -> Self {
        Self {
            span: Some(span),
            kind,
        }
    }

    fn single(at: usize, kind: ErrorKind) -> Self {
        Self {
            span: Some(at..at + 1),
            kind,
        }
    }

    fn spanless(kind: ErrorKind) -> Self {
        Self {
            span: None,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The input was an empty string
    Empty,

    /// An unexpected char was encountered.
    UnexpectedChar,

    /// Literal was not recognized.
    InvalidLiteral,

    /// Input does not start with decimal digit when trying to parse an integer.
    DoesNotStartWithDigit,

    /// Integer literal does not contain any valid digits.
    NoValidDigits,

    /// An integer literal overflows the target type.
    IntegerOverflow,

    /// Found a integer type suffix that is invalid.
    InvalidIntegerTypeSuffix,

    /// Found a float type suffix that is invalid. Only `f32` and `f64` are
    /// valid.
    InvalidFloatTypeSuffix,

    /// Exponent of a float literal does not contain any digits.
    NoExponentDigits,

    /// Something about an escape in a string, character, byte string or byte
    /// literal is broken.
    InvalidEscape,

    /// A string or character literal using the `\xNN` escape where `NN > 0x7F`.
    NonAsciiXEscape,

    /// A (raw) string, character, (raw) byte string or byte literal that's not
    /// terminated.
    UnterminatedLiteral,

    /// A character contains more than one character.
    OverlongCharLiteral,

    /// An empty character literal, i.e. `''`.
    EmptyCharLiteral,

    /// A `'` character was not escaped in a character or byte literal, or a `"`
    /// character was not escaped in a string or byte string literal.
    UnescapedQuote,

    /// When parsing a character, byte, string or byte string literal directly
    /// and the input does not start with the corresponding quote character
    /// (plus optional raw string prefix).
    DoesNotStartWithQuote,
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
