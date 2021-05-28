//! `From` and `TryFrom` impls for various conversions.
//!
//! # Tests
//!
//! ```no_run
//! #[cfg(not(feature = "proc-macro2"))]
//! compile_error!("Run tests with feature `proc-macro2` enabled!");
//!
//! extern crate proc_macro;
//!
//! use std::convert::TryFrom;
//! use litrs::Literal;
//!
//! fn give<T>() -> T {
//!     panic!()
//! }
//!
//! let _ = litrs::Literal::<String>::from(give::<litrs::BoolLit>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::IntegerLit<String>>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::FloatLit<String>>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::CharLit<String>>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::StringLit<String>>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::ByteLit<String>>());
//! let _ = litrs::Literal::<String>::from(give::<litrs::ByteStringLit<String>>());
//!
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::BoolLit>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::IntegerLit<&'static str>>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::FloatLit<&'static str>>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::CharLit<&'static str>>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::StringLit<&'static str>>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::ByteLit<&'static str>>());
//! let _ = litrs::Literal::<&'static str>::from(give::<litrs::ByteStringLit<&'static str>>());
//!
//!
//! let _ = litrs::Literal::from(give::<proc_macro::Literal>());
//! let _ = litrs::Literal::from(give::<&proc_macro::Literal>());
//! let _ = litrs::Literal::from(give::<proc_macro2::Literal>());
//! let _ = litrs::Literal::from(give::<&proc_macro2::Literal>());
//!
//! let _ = litrs::Literal::try_from(give::<proc_macro::TokenTree>());
//! let _ = litrs::Literal::try_from(give::<&proc_macro::TokenTree>());
//! let _ = litrs::Literal::try_from(give::<proc_macro2::TokenTree>());
//! let _ = litrs::Literal::try_from(give::<&proc_macro2::TokenTree>());
//! ```

use std::convert::TryFrom;

use crate::{Literal, err::{InvalidToken, TokenKind}};


// ==============================================================================================
// ===== `From<*Lit> for Literal`
// ==============================================================================================

macro_rules! impl_specific_lit_to_lit {
    ($ty:ty, $variant:ident) => {
        impl<B: crate::Buffer> From<$ty> for Literal<B> {
            fn from(src: $ty) -> Self {
                Literal::$variant(src)
            }
        }
    };
}

impl_specific_lit_to_lit!(crate::BoolLit, Bool);
impl_specific_lit_to_lit!(crate::IntegerLit<B>, Integer);
impl_specific_lit_to_lit!(crate::FloatLit<B>, Float);
impl_specific_lit_to_lit!(crate::CharLit<B>, Char);
impl_specific_lit_to_lit!(crate::StringLit<B>, String);
impl_specific_lit_to_lit!(crate::ByteLit<B>, Byte);
impl_specific_lit_to_lit!(crate::ByteStringLit<B>, ByteString);



// ==============================================================================================
// ===== `From<pm::Literal> for Literal`
// ==============================================================================================

// We call `expect` in all these impls: this library aims to implement exactly
// the Rust grammar, so if we have a valid Rust literal, we should always be
// able to parse it.
impl From<proc_macro::Literal> for Literal<String> {
    fn from(src: proc_macro::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

impl From<&proc_macro::Literal> for Literal<String> {
    fn from(src: &proc_macro::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Literal> for Literal<String> {
    fn from(src: proc_macro2::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}

#[cfg(feature = "proc-macro2")]
impl From<&proc_macro2::Literal> for Literal<String> {
    fn from(src: &proc_macro2::Literal) -> Self {
        Self::parse(src.to_string())
            .expect("bug: failed to parse output of `Literal::to_string`")
    }
}


// ==============================================================================================
// ===== `TryFrom<pm::TokenTree> for Literal`
// ==============================================================================================

macro_rules! lit_or_err {
    ($tt:expr, $krate:ident) => {
        match $tt {
            $krate::TokenTree::Group(_) => Err(TokenKind::Group),
            $krate::TokenTree::Ident(_) => Err(TokenKind::Ident),
            $krate::TokenTree::Punct(_) => Err(TokenKind::Punct),
            $krate::TokenTree::Literal(lit) => Ok(lit),
        }
    };
}

impl TryFrom<proc_macro::TokenTree> for Literal<String> {
    type Error = InvalidToken;
    fn try_from(tt: proc_macro::TokenTree) -> Result<Self, Self::Error> {
        lit_or_err!(&tt, proc_macro)
            .map_err(|actual|  InvalidToken {
                actual,
                expected: TokenKind::Literal,
                span: tt.span().into(),
            })
            .map(From::from)
    }
}

impl TryFrom<&proc_macro::TokenTree> for Literal<String> {
    type Error = InvalidToken;
    fn try_from(tt: &proc_macro::TokenTree) -> Result<Self, Self::Error> {
        lit_or_err!(&tt, proc_macro)
            .map_err(|actual|  InvalidToken {
                actual,
                expected: TokenKind::Literal,
                span: tt.span().into(),
            })
            .map(From::from)
    }
}

#[cfg(feature = "proc-macro2")]
impl TryFrom<proc_macro2::TokenTree> for Literal<String> {
    type Error = InvalidToken;
    fn try_from(tt: proc_macro2::TokenTree) -> Result<Self, Self::Error> {
        lit_or_err!(&tt, proc_macro2)
            .map_err(|actual|  InvalidToken {
                actual,
                expected: TokenKind::Literal,
                span: tt.span().into(),
            })
            .map(From::from)
    }
}

#[cfg(feature = "proc-macro2")]
impl TryFrom<&proc_macro2::TokenTree> for Literal<String> {
    type Error = InvalidToken;
    fn try_from(tt: &proc_macro2::TokenTree) -> Result<Self, Self::Error> {
        lit_or_err!(&tt, proc_macro2)
            .map_err(|actual|  InvalidToken {
                actual,
                expected: TokenKind::Literal,
                span: tt.span().into(),
            })
            .map(From::from)
    }
}
