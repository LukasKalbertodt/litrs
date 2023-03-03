use std::{fmt, str::FromStr};

use crate::{
    Buffer, ParseError,
    err::{perr, ParseErrorKind::*},
    parse::{first_byte_or_empty, hex_digit_value},
};


/// An integer literal, e.g. `27`, `0x7F`, `0b101010u8` or `5_000_000i64`.
///
/// An integer literal consists of an optional base prefix (`0b`, `0o`, `0x`),
/// the main part (digits and underscores), and an optional type suffix
/// (e.g. `u64` or `i8`). See [the reference][ref] for more information.
///
/// Note that integer literals are always positive: the grammar does not contain
/// the minus sign at all. The minus sign is just the unary negate operator,
/// not part of the literal. Which is interesting for cases like `- 128i8`:
/// here, the literal itself would overflow the specified type (`i8` cannot
/// represent 128). That's why in rustc, the literal overflow check is
/// performed as a lint after parsing, not during the lexing stage. Similarly,
/// [`IntegerLit::parse`] does not perform an overflow check.
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#integer-literals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct IntegerLit<B: Buffer> {
    raw: B,
    // First index of the main number part (after the base prefix).
    start_main_part: usize,
    // First index not part of the main number part.
    end_main_part: usize,
    base: IntegerBase,
    type_suffix: Option<IntegerType>,
}

/// The bases in which an integer can be specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}
impl<B: Buffer> IntegerLit<B> {
    /// Parses the input as an integer literal. Returns an error if the input is
    /// invalid or represents a different kind of literal.
    pub fn parse(input: B) -> Result<Self, ParseError> {
        match first_byte_or_empty(&input)? {
            digit @ b'0'..=b'9' => {
                // TODO: simplify once RFC 2528 is stabilized
                let IntegerLit {
                    start_main_part,
                    end_main_part,
                    base,
                    type_suffix,
                    ..
                } =  parse_impl(&input, digit)?;

                Ok(Self {
                    raw: input,
                    start_main_part,
                    end_main_part,
                    base,
                    type_suffix,
                })
            },
            _ => Err(perr(0, DoesNotStartWithDigit)),
        }
    }

    /// Performs the actual string to int conversion to obtain the integer
    /// value. The optional type suffix of the literal **is ignored by this
    /// method**. This means `N` does not need to match the type suffix!
    ///
    /// Returns `None` if the literal overflows `N`.
    pub fn value<N: FromIntegerLiteral>(&self) -> Option<N> {
        let base = match self.base {
            IntegerBase::Binary => N::from_small_number(2),
            IntegerBase::Octal => N::from_small_number(8),
            IntegerBase::Decimal => N::from_small_number(10),
            IntegerBase::Hexadecimal => N::from_small_number(16),
        };

        let mut acc = N::from_small_number(0);
        for digit in self.raw_main_part().bytes() {
            if digit == b'_' {
                continue;
            }

            // We don't actually need the base here: we already know this main
            // part only contains digits valid for the specified base.
            let digit = hex_digit_value(digit)
                .unwrap_or_else(|| unreachable!("bug: integer main part contains non-digit"));

            acc = acc.checked_mul(base)?;
            acc = acc.checked_add(N::from_small_number(digit))?;
        }

        Some(acc)
    }

    /// The base of this integer literal.
    pub fn base(&self) -> IntegerBase {
        self.base
    }

    /// The main part containing the digits and potentially `_`. Do not try to
    /// parse this directly as that would ignore the base!
    pub fn raw_main_part(&self) -> &str {
        &(*self.raw)[self.start_main_part..self.end_main_part]
    }

    /// The type suffix, if specified.
    pub fn type_suffix(&self) -> Option<IntegerType> {
        self.type_suffix
    }

    /// Returns the raw input that was passed to `parse`.
    pub fn raw_input(&self) -> &str {
        &self.raw
    }

    /// Returns the raw input that was passed to `parse`, potentially owned.
    pub fn into_raw_input(self) -> B {
        self.raw
    }
}

impl IntegerLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn to_owned(&self) -> IntegerLit<String> {
        IntegerLit {
            raw: self.raw.to_owned(),
            start_main_part: self.start_main_part,
            end_main_part: self.end_main_part,
            base: self.base,
            type_suffix: self.type_suffix,
        }
    }
}

impl<B: Buffer> fmt::Display for IntegerLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.raw)
    }
}

/// Integer literal types. *Implementation detail*.
///
/// Implemented for all integer literal types. This trait is sealed and cannot
/// be implemented outside of this crate. The trait's methods are implementation
/// detail of this library and are not subject to semver.
pub trait FromIntegerLiteral: self::sealed::Sealed + Copy {
    /// Creates itself from the given number. `n` is guaranteed to be `<= 16`.
    #[doc(hidden)]
    fn from_small_number(n: u8) -> Self;

    #[doc(hidden)]
    fn checked_add(self, rhs: Self) -> Option<Self>;

    #[doc(hidden)]
    fn checked_mul(self, rhs: Self) -> Option<Self>;

    #[doc(hidden)]
    fn ty() -> IntegerType;
}

macro_rules! impl_from_int_literal {
    ($( $ty:ty => $variant:ident ,)* ) => {
        $(
            impl self::sealed::Sealed for $ty {}
            impl FromIntegerLiteral for $ty {
                fn from_small_number(n: u8) -> Self {
                    n as Self
                }
                fn checked_add(self, rhs: Self) -> Option<Self> {
                    self.checked_add(rhs)
                }
                fn checked_mul(self, rhs: Self) -> Option<Self> {
                    self.checked_mul(rhs)
                }
                fn ty() -> IntegerType {
                    IntegerType::$variant
                }
            }
        )*
    };
}

impl_from_int_literal!(
    u8 => U8, u16 => U16, u32 => U32, u64 => U64, u128 => U128, usize => Usize,
    i8 => I8, i16 => I16, i32 => I32, i64 => I64, i128 => I128, isize => Isize,
);

mod sealed {
    pub trait Sealed {}
}

/// Precondition: first byte of string has to be in `b'0'..=b'9'`.
#[inline(never)]
pub(crate) fn parse_impl(input: &str, first: u8) -> Result<IntegerLit<&str>, ParseError> {
    // Figure out base and strip prefix base, if it exists.
    let (end_prefix, base) = match (first, input.as_bytes().get(1)) {
        (b'0', Some(b'b')) => (2, IntegerBase::Binary),
        (b'0', Some(b'o')) => (2, IntegerBase::Octal),
        (b'0', Some(b'x')) => (2, IntegerBase::Hexadecimal),

        // Everything else is treated as decimal. Several cases are caught
        // by this:
        // - "123"
        // - "0"
        // - "0u8"
        // - "0r" -> this will error later
        _ => (0, IntegerBase::Decimal),
    };
    let without_prefix = &input[end_prefix..];

    // Find end of main part.
    let end_main = without_prefix.bytes()
            .position(|b| !matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'_'))
            .unwrap_or(without_prefix.len());
    let (main_part, type_suffix) = without_prefix.split_at(end_main);

    // Check for invalid digits and make sure there is at least one valid digit.
    let invalid_digit_pos = match base {
        IntegerBase::Binary => main_part.bytes()
            .position(|b| !matches!(b, b'0' | b'1' | b'_')),
        IntegerBase::Octal => main_part.bytes()
            .position(|b| !matches!(b, b'0'..=b'7' | b'_')),
        IntegerBase::Decimal => main_part.bytes()
            .position(|b| !matches!(b, b'0'..=b'9' | b'_')),
        IntegerBase::Hexadecimal => None,
    };

    if let Some(pos) = invalid_digit_pos {
        return Err(perr(end_prefix + pos, InvalidDigit));
    }

    if main_part.bytes().filter(|&b| b != b'_').count() == 0 {
        return Err(perr(end_prefix..end_prefix + end_main, NoDigits));
    }


    // Parse type suffix
    let type_suffix = match type_suffix {
        "" => None,
        "u8" => Some(IntegerType::U8),
        "u16" => Some(IntegerType::U16),
        "u32" => Some(IntegerType::U32),
        "u64" => Some(IntegerType::U64),
        "u128" => Some(IntegerType::U128),
        "usize" => Some(IntegerType::Usize),
        "i8" => Some(IntegerType::I8),
        "i16" => Some(IntegerType::I16),
        "i32" => Some(IntegerType::I32),
        "i64" => Some(IntegerType::I64),
        "i128" => Some(IntegerType::I128),
        "isize" => Some(IntegerType::Isize),
        _ => return Err(perr(end_main + end_prefix..input.len(), InvalidIntegerTypeSuffix)),
    };

    Ok(IntegerLit {
        raw: input,
        start_main_part: end_prefix,
        end_main_part: end_main + end_prefix,
        base,
        type_suffix,
    })
}



/// All possible integer type suffixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

impl IntegerType {
    /// Returns the type corresponding to the given suffix (e.g. `"u8"` is
    /// mapped to `Self::U8`). If the suffix is not a valid integer type,
    /// `None` is returned.
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "u8" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "u128" => Some(Self::U128),
            "usize" => Some(Self::Usize),
            "i8" => Some(Self::I8),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "i128" => Some(Self::I128),
            "isize" => Some(Self::Isize),
            _ => None,
        }
    }

    /// Returns the suffix for this type, e.g. `"u8"` for `Self::U8`.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::Usize => "usize",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::Isize => "isize",
        }
    }
}

impl FromStr for IntegerType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_suffix(s).ok_or(())
    }
}

impl fmt::Display for IntegerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.suffix().fmt(f)
    }
}


#[cfg(test)]
mod tests;
