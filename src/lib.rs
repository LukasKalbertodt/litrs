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
mod string;


use std::{borrow::{Borrow, Cow}, fmt, ops::{Deref, Range}};

pub use self::{
    bool::BoolLit,
    char::CharLit,
    float::{FloatLit, FloatType},
    integer::{IntegerLit, IntegerBase, IntegerType},
    string::StringLit,
};


pub type OwnedLit = Literal<String>;
pub type SharedLit<'a> = Literal<&'a str>;


#[derive(Debug, Clone, PartialEq)]
pub enum Literal<B: Buffer> {
    Bool(BoolLit),
    Integer(IntegerLit<B>),
    Float(FloatLit<B>),
    Char(CharLit<B>),
    String(StringLit<B>),
    Byte,
    ByteString,
}


/// Errors during parsing.
///
/// This type should be seen primarily for error reporting and not for catching
/// specific cases. The span and error kind are not guaranteed to be stable
/// over different versions, meaning that a returned error can change between
/// versions of this library. There are simply too many fringe cases that are
/// not easy to classify as a specific error kind. It depends entirely on the
/// specific parser code how an invalid input is categorized.
///
/// Consider these examples:
/// - `'\` can be seen as
///     - invalid escape, or
///     - unterminated character literal.
/// - `'''` can be seen as
///     - empty character literal, or
///     - unescaped quote character.
/// - `0b64` can be seen as
///     - binary integer literal with invalid digit 6, or
///     - binary integer literal with invalid digit 4, or
///     - decimal integer literal with invalid digit b, or
///     - decimal integer literal 0 with unknown type suffix `b64`.
///
/// If you want to see more if these examples, feel free to check out the unit
/// tests of this library.
///
/// While this library does its best to emit sensible and precise errors, and to
/// keep the returned errors as stable as possible, full stability cannot be
/// guaranteed.
#[derive(Debug, Clone)]
pub struct Error {
    span: Option<Range<usize>>,
    kind: ErrorKind,
}

impl Error {
    /// Returns a span of this error, if available. **Note**: this is not
    /// stable. See[the documentation of this type][Error] for more
    /// information.
    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }

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

/// Kinds of errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
enum ErrorKind {
    /// The input was an empty string
    Empty,

    /// An unexpected char was encountered.
    UnexpectedChar,

    /// Literal was not recognized.
    InvalidLiteral,

    /// Input does not start with decimal digit when trying to parse an integer.
    DoesNotStartWithDigit,

    /// A digit invalid for the specified integer base was found.
    InvalidDigit,

    /// Integer literal does not contain any valid digits.
    NoDigits,

    /// Found a integer type suffix that is invalid.
    InvalidIntegerTypeSuffix,

    /// Found a float type suffix that is invalid. Only `f32` and `f64` are
    /// valid.
    InvalidFloatTypeSuffix,

    /// Exponent of a float literal does not contain any digits.
    NoExponentDigits,

    /// An unknown escape code, e.g. `\b`.
    UnknownEscape,

    /// A started escape sequence where the input ended before the escape was
    /// finished.
    UnterminatedEscape,

    /// An `\x` escape where the two digits are not valid hex digits.
    InvalidXEscape,

    /// A string or character literal using the `\xNN` escape where `NN > 0x7F`.
    NonAsciiXEscape,

    /// A `\u{...}` escape in a byte or byte string literal.
    UnicodeEscapeInByteLiteral,

    /// A Unicode escape that does not start with a hex digit.
    InvalidStartOfUnicodeEscape,

    /// A `\u{...}` escape that lacks the opening brace.
    UnicodeEscapeWithoutBrace,

    /// In a `\u{...}` escape, a non-hex digit and non-underscore character was
    /// found.
    NonHexDigitInUnicodeEscape,

    /// More than 6 digits found in unicode escape.
    TooManyDigitInUnicodeEscape,

    /// The value from a unicode escape does not represent a valid character.
    InvalidUnicodeEscapeChar,

    /// A `\u{..` escape that is not terminated (lacks the closing brace).
    UnterminatedUnicodeEscape,

    /// A character literal that's not terminated.
    UnterminatedCharLiteral,

    /// A character literal that contains more than one character.
    OverlongCharLiteral,

    /// An empty character literal, i.e. `''`.
    EmptyCharLiteral,

    /// A `'` character was not escaped in a character or byte literal, or a `"`
    /// character was not escaped in a string or byte string literal.
    UnescapedSingleQuote,

    /// When parsing a character, byte, string or byte string literal directly
    /// and the input does not start with the corresponding quote character
    /// (plus optional raw string prefix).
    DoesNotStartWithQuote,

    /// Unterminated raw string literal.
    UnterminatedRawString,

    /// Invalid start for a string literal.
    InvalidStringLiteralStart,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrorKind::*;

        let description = match self.kind {
            Empty => "input is empty",
            UnexpectedChar => "unexpected character",
            InvalidLiteral => "invalid literal",
            DoesNotStartWithDigit => "number literal does not start with decimal digit",
            InvalidDigit => "integer literal contains a digit invalid for its base",
            NoDigits => "integer literal does not contain any digits",
            InvalidIntegerTypeSuffix => "invalid integer type suffix",
            InvalidFloatTypeSuffix => "invalid floating point type suffix",
            NoExponentDigits => "exponent of floating point literal does not contain any digits",
            UnknownEscape => "unknown escape",
            UnterminatedEscape => "unterminated escape: input ended too soon",
            InvalidXEscape => r"invalid `\x` escape: not followed by two hex digits",
            NonAsciiXEscape => r"`\x` escape in char/string literal exceed ASCII range",
            UnicodeEscapeInByteLiteral => r"`\u{...}` escape in byte (string) literal not allowed",
            InvalidStartOfUnicodeEscape => r"invalid start of `\u{...}` escape",
            UnicodeEscapeWithoutBrace => r"`Unicode \u{...}` escape without opening brace",
            NonHexDigitInUnicodeEscape => r"non-hex digit found in `\u{...}` escape",
            TooManyDigitInUnicodeEscape => r"more than six digits in `\u{...}` escape",
            InvalidUnicodeEscapeChar => r"value specified in `\u{...}` escape is not a valid char",
            UnterminatedUnicodeEscape => r"unterminated `\u{...}` escape",
            UnterminatedCharLiteral => "character literal is not terminated",
            OverlongCharLiteral => "character literal contains more than one character",
            EmptyCharLiteral => "empty character literal",
            UnescapedSingleQuote => "character literal contains unescaped ' character",
            DoesNotStartWithQuote => "invalid start for char/byte/string literal",
            UnterminatedRawString => "unterminated raw string literal",
            InvalidStringLiteralStart => "invalid start for string literal",
        };

        description.fmt(f)?;
        if let Some(span) = &self.span {
            write!(f, " (at {}..{})", span.start, span.end)?;
        }

        Ok(())
    }
}

/// A shared or owned string buffer, implemented for `String` and `&str`.
///
/// This is trait is implementation detail of this library, cannot be
/// implemented in other crates and is not subject to semantic versioning.
/// `litrs` only gurantees that this trait is implemented for `String` and
/// `&str`.
pub trait Buffer: sealed::Sealed + Deref<Target = str> {
    /// This is `Cow<'static, str>` for `String, and `Cow<'a, str>` for `&'a str`.
    type Cow: From<String> + AsRef<str> + Borrow<str> + Deref<Target = str>;

    #[doc(hidden)]
    fn into_cow(self) -> Self::Cow;

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

    type Cow = Cow<'a, str>;
    fn into_cow(self) -> Self::Cow {
        self.into()
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

    type Cow = Cow<'static, str>;
    fn into_cow(self) -> Self::Cow {
        self.into()
    }
}
