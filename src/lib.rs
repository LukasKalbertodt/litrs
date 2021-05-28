//! Parsing and inspecting Rust literal tokens.
//!
//! This library offers functionality to parse Rust literals, i.e. tokens in the
//! Rust programming language that represent fixed values. The grammar for
//! those is defined [here][ref].
//!
//! This kind of functionality already exists in the crate `syn`. However, as
//! you oftentimes don't need (nor want) the full power of `syn`, `litrs` was
//! built. This crate also offers a bit more flexibility compared to `syn`
//! (only regarding literals, of course).
//!
//! ---
//!
//! The main types of this library are [`Literal`], representing any kind of
//! literal, and `*Lit`, like [`StringLit`] or [`FloatLit`], representing a
//! specific kind of literal.
//!
//! There are different ways to obtain such a literal type:
//!
//! - **`parse`**: parses a `&str` or `String` and returns `Result<_,
//!     ParseError>`. For example: [`Literal::parse`] and
//!     [`IntegerLit::parse`].
//!
//! - **`From<proc_macro::Literal> for Literal`**: turns a `Literal` value from
//!     the `proc_macro` crate into a `Literal` from this crate.
//!
//! - **`TryFrom<proc_macro::Literal> for *Lit`**: tries to turn a
//!     `proc_macro::Literal` into a specific literal type of this crate. If
//!     the input is a literal of a different kind, `Err(InvalidToken)` is
//!     returned.
//!
//! - **`TryFrom<proc_macro::TokenTree>`**: attempts to turn a token tree into a
//!     literal type of this crate. An error is returned if the token tree is
//!     not a literal, or if you are trying to turn it into a specific kind of
//!     literal and the token tree is a different kind of literal.
//!
//! All of the `From` and `TryFrom` conversions also work for reference to
//! `proc_macro` types. Additionally, if the crate feature `proc-macro2` is
//! enabled (which it is by default), all these `From` and `TryFrom` impls also
//! exist for the corresponding `proc_macro2` types.
//!
//! **Note**: `true` and `false` are `Ident`s when passed to your proc macro.
//! The `TryFrom<TokenTree>` impls check for those two special idents and
//! return a `BoolLit` appropriately. For that reason, there is also no
//! `TryFrom<proc_macro::Literal>` impl for `BoolLit`. The `proc_macro::Literal`
//! simply cannot represent bool literals.
//!
//!
//! # Examples
//!
//! In a proc-macro:
//!
//! ```ignore
//! use std::convert::TryFrom;
//! use proc_macro::TokenStream;
//! use litrs::FloatLit;
//!
//! #[proc_macro]
//! pub fn foo(input: TokenStream) -> TokenStream {
//!      let mut input = input.into_iter().collect::<Vec<_>>();
//!      if input.len() != 1 {
//!          // Please do proper error handling in your real code!
//!          panic!("expected exactly one token as input");
//!      }
//!      let token = input.remove(0);
//!
//!      match FloatLit::try_from(token) {
//!          Ok(float_lit) => { /* do something */ }
//!          Err(e) => return e.to_compile_error(),
//!      }
//!
//!      // Dummy output
//!      TokenStream::new()
//! }
//! ```
//!
//! Parsing from string:
//!
//! ```
//! use litrs::{FloatLit, Literal};
//!
//! // Parse a specific kind of literal (float in this case):
//! let float_lit = FloatLit::parse("3.14f32");
//! assert!(float_lit.is_ok());
//! assert_eq!(float_lit.unwrap().type_suffix(), Some(litrs::FloatType::F32));
//! assert!(FloatLit::parse("'c'").is_err());
//!
//! // Parse any kind of literal. After parsing, you can inspect the literal
//! // and decide what to do in each case.
//! let lit = Literal::parse("0xff80").expect("failed to parse literal");
//! match lit {
//!     Literal::Integer(lit) => { /* ... */ }
//!     Literal::Float(lit) => { /* ... */ }
//!     Literal::Bool(lit) => { /* ... */ }
//!     Literal::Char(lit) => { /* ... */ }
//!     Literal::String(lit) => { /* ... */ }
//!     Literal::Byte(lit) => { /* ... */ }
//!     Literal::ByteString(lit) => { /* ... */ }
//! }
//! ```
//!
//!
//!
//! # Crate features
//!
//! - `proc-macro2` (**default**): adds the dependency `proc_macro2`, a bunch of
//!   `From` and `TryFrom` impls, and [`InvalidToken::to_compile_error2`].
//!
//!
//! [ref]: https://doc.rust-lang.org/reference/tokens.html#literals
//!

#![deny(missing_debug_implementations)]

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
mod err;
mod escape;
mod float;
mod impls;
mod integer;
mod parse;
mod string;


use std::{borrow::{Borrow, Cow}, fmt, ops::{Deref, Range}};

pub use self::{
    bool::BoolLit,
    byte::ByteLit,
    bytestr::ByteStringLit,
    char::CharLit,
    err::{InvalidToken, ParseError},
    float::{FloatLit, FloatType},
    integer::{FromIntegerLiteral, IntegerLit, IntegerBase, IntegerType},
    string::StringLit,
};


// ==============================================================================================
// ===== `Literal` and type defs
// ==============================================================================================

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
/// input string or use the `From<_>` impls of this type. The impls are only
/// available of the corresponding crate features are enabled (they are enabled
/// by default).
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


// ==============================================================================================
// ===== Buffer
// ==============================================================================================

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
