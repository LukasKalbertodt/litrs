extern crate proc_macro;

#[cfg(test)]
#[macro_use]
mod test_util;

#[cfg(test)]
mod tests;

mod bool;
mod byte;
mod bytestr;
mod char;
mod escape;
mod float;
mod integer;
mod parse;
mod string;


use std::{borrow::{Borrow, Cow}, fmt, ops::{Deref, Range}};

pub use self::{
    bool::BoolLit,
    byte::ByteLit,
    bytestr::ByteStringLit,
    char::CharLit,
    float::{FloatLit, FloatType},
    integer::{FromIntegerLiteral, IntegerLit, IntegerBase, IntegerType},
    string::StringLit,
};


/// A literal which owns the underlying buffer.
pub type OwnedLiteral = Literal<String>;

/// A literal whose underlying buffer is borrowed.
pub type SharedLiteral<'a> = Literal<&'a str>;

/// A literal. This is the main type of this library.
///
/// This type is generic over the underlying buffer `B`, which can be `&str` or
/// `String`. There are two useful type aliases: [`OwnedLiteral`] and
/// [`SharedLiteral`].
///
/// To create this type, you have to either call [`Literal::parse`] with an
/// input string or use the `From<proc_macro[2]::Literal>` impls. The impls are
/// only available of the corresponding crate features are enabled (they are
/// enabled by default).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal<B: Buffer> {
    Bool(BoolLit),
    Integer(IntegerLit<B>),
    Float(FloatLit<B>),
    Char(CharLit<B>),
    String(StringLit<B>),
    Byte(ByteLit<B>),
    ByteString(ByteStringLit<B>),
}

impl Literal<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn into_owned(self) -> OwnedLiteral {
        match self {
            Literal::Bool(l) => Literal::Bool(l.to_owned()),
            Literal::Integer(l) => Literal::Integer(l.to_owned()),
            Literal::Float(l) => Literal::Float(l.to_owned()),
            Literal::Char(l) => Literal::Char(l.to_owned()),
            Literal::String(l) => Literal::String(l.into_owned()),
            Literal::Byte(l) => Literal::Byte(l.to_owned()),
            Literal::ByteString(l) => Literal::ByteString(l.into_owned()),
        }
    }
}

impl<B: Buffer> fmt::Display for Literal<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Bool(l) => l.fmt(f),
            Literal::Integer(l) => l.fmt(f),
            Literal::Float(l) => l.fmt(f),
            Literal::Char(l) => l.fmt(f),
            Literal::String(l) => l.fmt(f),
            Literal::Byte(l) => l.fmt(f),
            Literal::ByteString(l) => l.fmt(f),
        }
    }
}

#[cfg(feature = "proc-macro")]
impl From<proc_macro::Literal> for Literal<String> {
    fn from(src: proc_macro::Literal) -> Self {
        // This library aims to implement exactly the Rust grammar, so if we
        // have a valid Rust literal, we should always be able to parse it.
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Literal> for Literal<String> {
    fn from(src: proc_macro2::Literal) -> Self {
        // This library aims to implement exactly the Rust grammar, so if we
        // have a valid Rust literal, we should always be able to parse it.
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}


/// Errors during parsing.
///
/// This type should be seen primarily for error reporting and not for catching
/// specific cases. The span and error kind are not guaranteed to be stable
/// over different versions of this library, meaning that a returned error can
/// change from one version to the next. There are simply too many fringe cases
/// that are not easy to classify as a specific error kind. It depends entirely
/// on the specific parser code how an invalid input is categorized.
///
/// Consider these examples:
/// - `'\` can be seen as
///     - invalid escape in character literal, or
///     - unterminated character literal.
/// - `'''` can be seen as
///     - empty character literal, or
///     - unescaped quote character in character literal.
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
    /// stable. See [the documentation of this type][Error] for more
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

    UnterminatedByteLiteral,
    OverlongByteLiteral,
    EmptyByteLiteral,
    NonAsciiInByteLiteral,

    /// A `'` character was not escaped in a character or byte literal, or a `"`
    /// character was not escaped in a string or byte string literal.
    UnescapedSingleQuote,

    /// A \n, \t or \r raw character in a char or byte literal.
    UnescapedSpecialWhitespace,

    /// When parsing a character, byte, string or byte string literal directly
    /// and the input does not start with the corresponding quote character
    /// (plus optional raw string prefix).
    DoesNotStartWithQuote,

    /// Unterminated raw string literal.
    UnterminatedRawString,

    /// String literal without a `"` at the end.
    UnterminatedString,

    /// Invalid start for a string literal.
    InvalidStringLiteralStart,

    /// Invalid start for a byte literal.
    InvalidByteLiteralStart,

    InvalidByteStringLiteralStart,

    /// An literal `\r` character not followed by a `\n` character in a
    /// (raw) string or byte string literal.
    IsolatedCr,
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
            UnterminatedByteLiteral => "byte literal is not terminated",
            OverlongByteLiteral => "byte literal contains more than one byte",
            EmptyByteLiteral => "empty byte literal",
            NonAsciiInByteLiteral => "non ASCII character in byte (string) literal",
            UnescapedSingleQuote => "character literal contains unescaped ' character",
            UnescapedSpecialWhitespace => r"unescaped newline (\n), tab (\t) or cr (\r) character",
            DoesNotStartWithQuote => "invalid start for char/byte/string literal",
            UnterminatedRawString => "unterminated raw (byte) string literal",
            UnterminatedString => "unterminated (byte) string literal",
            InvalidStringLiteralStart => "invalid start for string literal",
            InvalidByteLiteralStart => "invalid start for byte literal",
            InvalidByteStringLiteralStart => "invalid start for byte string literal",
            IsolatedCr => r"`\r` not immediately followed by `\n` in string",
        };

        description.fmt(f)?;
        if let Some(span) = &self.span {
            write!(f, " (at {}..{})", span.start, span.end)?;
        }

        Ok(())
    }
}

/// A shared or owned string buffer. Implemented for `String` and `&str`. *Implementation detail*.
///
/// This is trait is implementation detail of this library, cannot be
/// implemented in other crates and is not subject to semantic versioning.
/// `litrs` only gurantees that this trait is implemented for `String` and
/// `for<'a> &'a str`.
pub trait Buffer: sealed::Sealed + Deref<Target = str> {
    /// This is `Cow<'static, str>` for `String`, and `Cow<'a, str>` for `&'a str`.
    type Cow: From<String> + AsRef<str> + Borrow<str> + Deref<Target = str>;

    #[doc(hidden)]
    fn into_cow(self) -> Self::Cow;

    /// This is `Cow<'static, [u8]>` for `String`, and `Cow<'a, [u8]>` for `&'a str`.
    type ByteCow: From<Vec<u8>> + AsRef<[u8]> + Borrow<[u8]> + Deref<Target = [u8]>;

    #[doc(hidden)]
    fn into_byte_cow(self) -> Self::ByteCow;

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
    #[doc(hidden)]
    fn cut(self, range: Range<usize>) -> Self {
        &self[range]
    }

    type Cow = Cow<'a, str>;
    #[doc(hidden)]
    fn into_cow(self) -> Self::Cow {
        self.into()
    }
    type ByteCow = Cow<'a, [u8]>;
    #[doc(hidden)]
    fn into_byte_cow(self) -> Self::ByteCow {
        self.as_bytes().into()
    }
}

impl sealed::Sealed for String {}
impl Buffer for String {
    #[doc(hidden)]
    fn cut(mut self, range: Range<usize>) -> Self {
        // This is not the most efficient way, but it works. First we cut the
        // end, then the beginning. Note that `drain` also removes the range if
        // the iterator is not consumed.
        self.truncate(range.end);
        self.drain(..range.start);
        self
    }

    type Cow = Cow<'static, str>;
    #[doc(hidden)]
    fn into_cow(self) -> Self::Cow {
        self.into()
    }

    type ByteCow = Cow<'static, [u8]>;
    #[doc(hidden)]
    fn into_byte_cow(self) -> Self::ByteCow {
        self.into_bytes().into()
    }
}
