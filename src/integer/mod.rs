use crate::{Buffer, Error, parse::{first_byte_or_empty, hex_digit_value}};


/// An integer literal consisting of an optional base prefix (`0b`, `0o`, `0x`),
/// the main part containing digits and optional `_`, and an optional type
/// suffix (e.g. `u64` or `i8`).
///
/// Note that integer literals are always positive: the grammar does not contain
/// the minus sign at all. The minus sign is just the unary negate operator, not
/// part of the literal. Which is interesting for cases like `- 128i8`: here,
/// the literal itself would overflow the specified type (`i8` cannot represent
/// 128). That's why in rustc, the literal overflow check is performed as a lint
/// after parsing, not during the lexing stage. Similarly, `Integer::parse` does
/// not perform an overflow check.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Integer<B: Buffer> {
    base: IntegerBase,
    main_part: B,
    type_suffix: Option<IntegerType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

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

impl IntegerBase {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Binary => "0b",
            Self::Octal => "0o",
            Self::Decimal => "",
            Self::Hexadecimal => "0x",
        }
    }
}

impl<B: Buffer> Integer<B> {
    pub fn parse(input: B) -> Result<Self, Error> {
        match first_byte_or_empty(&input)? {
            digit @ b'0'..=b'9' => Self::parse_impl(input, digit),
            _ => Err(Error::DoesNotStartWithDigit),
        }
    }

    /// Performs the actual string to int conversion to obtain the integer
    /// value. The optional type suffix of the literal **is ignored by this
    /// method**.
    ///
    /// Returns `None` if the literal overflows `N`.
    pub fn value<N: FromIntLiteral>(&self) -> Option<N> {
        let base = match self.base {
            IntegerBase::Binary => N::from_small_number(2),
            IntegerBase::Octal => N::from_small_number(8),
            IntegerBase::Decimal => N::from_small_number(10),
            IntegerBase::Hexadecimal => N::from_small_number(16),
        };

        let mut acc = N::from_small_number(0);
        for digit in self.main_part.bytes() {
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
        &self.main_part
    }

    /// The type suffix, if specified.
    pub fn type_suffix(&self) -> Option<IntegerType> {
        self.type_suffix
    }

    /// Precondition: first byte of string has to be in `b'0'..=b'9'`.
    pub(crate) fn parse_impl(input: B, first: u8) -> Result<Self, Error> {
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
        let without_prefix = &(*input)[end_prefix..];

        // Find end of main part.
        let end_main = match base {
            IntegerBase::Binary => without_prefix.bytes()
                .position(|b| !matches!(b, b'0' | b'1' | b'_')),
            IntegerBase::Octal => without_prefix.bytes()
                .position(|b| !matches!(b, b'0'..=b'7' | b'_')),
            IntegerBase::Decimal => without_prefix.bytes()
                .position(|b| !matches!(b, b'0'..=b'9' | b'_')),
            IntegerBase::Hexadecimal => without_prefix.bytes()
                .position(|b| !matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'_')),
        };
        let end_main = end_main.unwrap_or(without_prefix.len());
        let (main_part, type_suffix) = without_prefix.split_at(end_main);

        if main_part.bytes().filter(|&b| b != b'_').count() == 0 {
            return Err(Error::NoValidDigits);
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
            _ => return Err(Error::InvalidIntegerTypeSuffix {
                offset: main_part.len() + base.prefix().len(),
            }),
        };

        Ok(Self {
            base,
            main_part: input.cut(end_prefix..end_main + end_prefix),
            type_suffix,
        })
    }
}

/// Implemented for all integer literal types. This trait is sealed and cannot
/// be implemented outside of this crate. The trait's methods are implementation
/// detail of this library and are not subject to semver.
pub trait FromIntLiteral: self::sealed::Sealed + Copy {
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
            impl FromIntLiteral for $ty {
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


#[cfg(test)]
mod tests;
