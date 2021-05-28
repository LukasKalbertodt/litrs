use std::{fmt, ops::Range};


#[derive(Debug, Clone, Copy)]
pub struct InvalidToken {
    pub(crate) expected: TokenKind,
    pub(crate) actual: TokenKind,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Punct,
    Ident,
    Group,
    Literal,
}

/// Unfortunately, we have to deal with both cases.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Span {
    One(proc_macro::Span),
    #[cfg(feature = "proc-macro2")]
    Two(proc_macro2::Span),
}

impl From<proc_macro::Span> for Span {
    fn from(src: proc_macro::Span) -> Self {
        Self::One(src)
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Span> for Span {
    fn from(src: proc_macro2::Span) -> Self {
        Self::Two(src)
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
pub struct ParseError {
    pub(crate) span: Option<Range<usize>>,
    pub(crate) kind: ParseErrorKind,
}

impl ParseError {
    /// Returns a span of this error, if available. **Note**: the returned span
    /// might change in future versions of this library. See [the documentation
    /// of this type][Error] for more information.
    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

/// This is a free standing function instead of an associated one to reduce
/// noise around parsing code. There are lots of places that create errors, we
/// I wanna keep them as short as possible.
pub(crate) fn perr(span: impl SpanLike, kind: ParseErrorKind) -> ParseError {
    ParseError {
        span: span.into_span(),
        kind,
    }
}

pub(crate) trait SpanLike {
    fn into_span(self) -> Option<Range<usize>>;
}

impl SpanLike for Option<Range<usize>> {
    fn into_span(self) -> Option<Range<usize>> {
        self
    }
}
impl SpanLike for Range<usize> {
    fn into_span(self) -> Option<Range<usize>> {
        Some(self)
    }
}
impl SpanLike for usize {
    fn into_span(self) -> Option<Range<usize>> {
        Some(self..self + 1)
    }
}


/// Kinds of errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum ParseErrorKind {
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

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;

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