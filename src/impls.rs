use std::convert::TryFrom;

use crate::{Literal, err::{InvalidToken, TokenKind}};


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
