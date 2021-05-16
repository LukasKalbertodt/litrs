use crate::{
    Error,
    parse::first_byte_or_empty,
};


#[derive(Debug, Clone, PartialEq)]
pub struct Integer<'a> {
    base: IntegerBase,
    main_part: &'a str,
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

impl<'a> Integer<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        match first_byte_or_empty(s)? {
            digit @ b'0'..=b'9' => Self::parse_impl(s, digit),
            _ => Err(Error::DoesNotStartWithDigit),
        }
    }

    /// Precondition: first byte of string has to be in `b'0'..=b'9'`.
    pub(crate) fn parse_impl(input: &'a str, first: u8) -> Result<Self, Error> {
        // Figure out base and strip prefix base, if it exists.
        let (without_prefix, base) = match (first, input.as_bytes().get(1)) {
            (b'0', Some(b'b')) => (&input[2..], IntegerBase::Binary),
            (b'0', Some(b'o')) => (&input[2..], IntegerBase::Octal),
            (b'0', Some(b'x')) => (&input[2..], IntegerBase::Hexadecimal),

            // Everything else is treated as decimal. Several cases are caught
            // by this:
            // - "123"
            // - "0"
            // - "0u8"
            // - "0r" -> this will error later
            _ => (input, IntegerBase::Decimal),
        };

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
            _ => return Err(Error::UnexpectedChar {
                // We know it's not empty: we checked above.
                c: type_suffix.chars().next().unwrap(),
                offset: main_part.len() + base.prefix().len(),
            }),
        };

        Ok(Self {
            base,
            main_part,
            type_suffix,
        })
    }
}


#[cfg(test)]
mod tests;
