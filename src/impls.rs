use std::convert::TryFrom;

use crate::{Literal, err::{InvalidToken, TokenKind}};


/// Helper macro to call a `callback` macro four times for all combinations of
/// `proc_macro`/`proc_macro2` and `&`/owned.
macro_rules! helper {
    ($callback:ident, $($input:tt)*) => {
        $callback!([proc_macro::] => $($input)*);
        $callback!([&proc_macro::] => $($input)*);
        #[cfg(feature = "proc-macro2")]
        $callback!([proc_macro2::] => $($input)*);
        #[cfg(feature = "proc-macro2")]
        $callback!([&proc_macro2::] => $($input)*);
    };
}


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


macro_rules! impl_tt_to_lit {
    ([$($prefix:tt)*] => ) => {
        impl From<$($prefix)* Literal> for Literal<String> {
            fn from(src: $($prefix)* Literal) -> Self {
                // We call `expect` in all these impls: this library aims to implement exactly
                // the Rust grammar, so if we have a valid Rust literal, we should always be
                // able to parse it.
                Self::parse(src.to_string())
                    .expect("bug: failed to parse output of `Literal::to_string`")
            }
        }
    }
}

helper!(impl_tt_to_lit, );


// ==============================================================================================
// ===== `TryFrom<pm::TokenTree> for Literal`
// ==============================================================================================

macro_rules! impl_tt_to_lit {
    ([$($prefix:tt)*] => ) => {
        impl TryFrom<$($prefix)* TokenTree> for Literal<String> {
            type Error = InvalidToken;
            fn try_from(tt: $($prefix)* TokenTree) -> Result<Self, Self::Error> {
                let span = tt.span();
                let res = match tt {
                    $($prefix)* TokenTree::Group(_) => Err(TokenKind::Group),
                    $($prefix)* TokenTree::Punct(_) => Err(TokenKind::Punct),
                    $($prefix)* TokenTree::Ident(ref ident) if ident.to_string() == "true"
                        => return Ok(Literal::Bool(crate::BoolLit::True)),
                    $($prefix)* TokenTree::Ident(ref ident) if ident.to_string() == "false"
                        => return Ok(Literal::Bool(crate::BoolLit::False)),
                    $($prefix)* TokenTree::Ident(_) => Err(TokenKind::Ident),
                    $($prefix)* TokenTree::Literal(ref lit) => Ok(lit),
                };

                match res {
                    Ok(lit) => Ok(From::from(lit)),
                    Err(actual) => Err(InvalidToken {
                        actual,
                        expected: TokenKind::Literal,
                        span: span.into(),
                    }),
                }
            }
        }
    }
}

helper!(impl_tt_to_lit, );


// ==============================================================================================
// ===== `TryFrom<pm::Literal> for *Lit` and `TryFrom<pm::TokenTree> for *Lit`
// ==============================================================================================

fn kind_of(lit: &Literal<String>) -> TokenKind {
    match lit {
        Literal::String(_) => TokenKind::StringLit,
        Literal::Bool(_) => TokenKind::BoolLit,
        Literal::Integer(_) => TokenKind::IntegerLit,
        Literal::Float(_) => TokenKind::FloatLit,
        Literal::Char(_) => TokenKind::CharLit,
        Literal::Byte(_) => TokenKind::ByteLit,
        Literal::ByteString(_) => TokenKind::ByteStringLit,
    }
}

macro_rules! impl_for_specific_lit {
    ([$($prefix:tt)*] => $ty:ty, $variant:ident, $kind:ident) => {
        impl TryFrom<$($prefix)* Literal> for $ty {
            type Error = InvalidToken;
            fn try_from(src: $($prefix)* Literal) -> Result<Self, Self::Error> {
                let span = src.span();
                let lit: Literal<String> = src.into();
                match lit {
                    Literal::$variant(s) => Ok(s),
                    other => Err(InvalidToken {
                        expected: TokenKind::$kind,
                        actual: kind_of(&other),
                        span: span.into(),
                    }),
                }
            }
        }

        impl TryFrom<$($prefix)* TokenTree> for $ty {
            type Error = InvalidToken;
            fn try_from(tt: $($prefix)* TokenTree) -> Result<Self, Self::Error> {
                let span = tt.span();
                let res = match tt {
                    $($prefix)* TokenTree::Group(_) => Err(TokenKind::Group),
                    $($prefix)* TokenTree::Punct(_) => Err(TokenKind::Punct),
                    $($prefix)* TokenTree::Ident(_) => Err(TokenKind::Ident),
                    $($prefix)* TokenTree::Literal(ref lit) => Ok(lit),
                };

                match res {
                    Ok(lit) => <$ty>::try_from(lit),
                    Err(actual) => Err(InvalidToken {
                        actual,
                        expected: TokenKind::$kind,
                        span: span.into(),
                    }),
                }
            }
        }
    };
}

helper!(impl_for_specific_lit, crate::IntegerLit<String>, Integer, IntegerLit);
helper!(impl_for_specific_lit, crate::FloatLit<String>, Float, FloatLit);
helper!(impl_for_specific_lit, crate::CharLit<String>, Char, CharLit);
helper!(impl_for_specific_lit, crate::StringLit<String>, String, StringLit);
helper!(impl_for_specific_lit, crate::ByteLit<String>, Byte, ByteLit);
helper!(impl_for_specific_lit, crate::ByteStringLit<String>, ByteString, ByteStringLit);

macro_rules! impl_from_tt_for_bool {
    ([$($prefix:tt)*] => ) => {
        impl TryFrom<$($prefix)* TokenTree> for crate::BoolLit {
            type Error = InvalidToken;
            fn try_from(tt: $($prefix)* TokenTree) -> Result<Self, Self::Error> {
                let span = tt.span();
                let actual = match tt {
                    $($prefix)* TokenTree::Ident(ref ident) if ident.to_string() == "true"
                        => return Ok(crate::BoolLit::True),
                    $($prefix)* TokenTree::Ident(ref ident) if ident.to_string() == "false"
                        => return Ok(crate::BoolLit::False),

                    $($prefix)* TokenTree::Group(_) => TokenKind::Group,
                    $($prefix)* TokenTree::Punct(_) => TokenKind::Punct,
                    $($prefix)* TokenTree::Ident(_) => TokenKind::Ident,
                    $($prefix)* TokenTree::Literal(ref lit) => kind_of(&Literal::from(lit)),
                };

                Err(InvalidToken {
                    actual,
                    expected: TokenKind::BoolLit,
                    span: span.into(),
                })
            }
        }
    };
}

helper!(impl_from_tt_for_bool, );


mod tests {
    //! # Tests
    //!
    //! ```no_run
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
    //!
    //! let _ = litrs::Literal::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::Literal::try_from(give::<&proc_macro::TokenTree>());
    //!
    //!
    //! let _ = litrs::IntegerLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::IntegerLit::try_from(give::<&proc_macro::Literal>());
    //!
    //! let _ = litrs::FloatLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::FloatLit::try_from(give::<&proc_macro::Literal>());
    //!
    //! let _ = litrs::CharLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::CharLit::try_from(give::<&proc_macro::Literal>());
    //!
    //! let _ = litrs::StringLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::StringLit::try_from(give::<&proc_macro::Literal>());
    //!
    //! let _ = litrs::ByteLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::ByteLit::try_from(give::<&proc_macro::Literal>());
    //!
    //! let _ = litrs::ByteStringLit::try_from(give::<proc_macro::Literal>());
    //! let _ = litrs::ByteStringLit::try_from(give::<&proc_macro::Literal>());
    //!
    //!
    //! let _ = litrs::BoolLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::BoolLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::IntegerLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::IntegerLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::FloatLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::FloatLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::CharLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::CharLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::StringLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::StringLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::ByteLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::ByteLit::try_from(give::<&proc_macro::TokenTree>());
    //!
    //! let _ = litrs::ByteStringLit::try_from(give::<proc_macro::TokenTree>());
    //! let _ = litrs::ByteStringLit::try_from(give::<&proc_macro::TokenTree>());
    //! ```
}

#[cfg(feature = "proc-macro2")]
mod tests_proc_macro2 {
    //! # Tests
    //!
    //! ```no_run
    //! extern crate proc_macro;
    //!
    //! use std::convert::TryFrom;
    //! use litrs::Literal;
    //!
    //! fn give<T>() -> T {
    //!     panic!()
    //! }
    //!
    //! let _ = litrs::Literal::from(give::<proc_macro2::Literal>());
    //! let _ = litrs::Literal::from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::Literal::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::Literal::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //!
    //! let _ = litrs::IntegerLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::IntegerLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::FloatLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::FloatLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::CharLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::CharLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::StringLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::StringLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::ByteLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::ByteLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //! let _ = litrs::ByteStringLit::try_from(give::<proc_macro2::Literal>());
    //! let _ = litrs::ByteStringLit::try_from(give::<&proc_macro2::Literal>());
    //!
    //!
    //! let _ = litrs::BoolLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::BoolLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::IntegerLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::IntegerLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::FloatLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::FloatLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::CharLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::CharLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::StringLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::StringLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::ByteLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::ByteLit::try_from(give::<&proc_macro2::TokenTree>());
    //!
    //! let _ = litrs::ByteStringLit::try_from(give::<proc_macro2::TokenTree>());
    //! let _ = litrs::ByteStringLit::try_from(give::<&proc_macro2::TokenTree>());
    //! ```
}
