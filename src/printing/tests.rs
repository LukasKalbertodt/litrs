use proc_macro2::{Literal, TokenTree};
use quote::ToTokens;

use crate::{ByteLit, ByteStringLit, CharLit, FloatLit, IntegerLit, StringLit};

#[test]
fn it_preserves_bool_true() {
    let other = crate::BoolLit::True
        .to_token_stream()
        .into_iter()
        .next()
        .and_then(|v| {
            if let TokenTree::Ident(v) = v {
                Some(v)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(other, "true")
}
#[test]
fn it_preserves_bool_false() {
    let other = crate::BoolLit::False
        .to_token_stream()
        .into_iter()
        .next()
        .and_then(|v| {
            if let TokenTree::Ident(v) = v {
                Some(v)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(other, "false")
}

// NOTE: sometimes the round-trip to quote simplifies literals (for example float literals)
// this is something that has to be taken into consideration when writing tests
macro_rules! preservation {
    ($name:ident : $ty:ident : $($content:tt)+) => {
        #[test]
        fn $name() {
            use std::convert::TryFrom;

            let lit = Literal::$($content)+;

            let lhs = $ty::try_from(lit).unwrap();

            let rhs = $ty::try_from(
                lhs.to_token_stream()
                    .into_iter()
                    .next()
                    .and_then(|v| {
                        if let TokenTree::Literal(v) = v {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

            assert_eq!(lhs, rhs);
        }
    }
}

preservation!(it_preserves_u8_suffixed   : IntegerLit :   u8_suffixed(10));
preservation!(it_preserves_u8_unsuffixed : IntegerLit : u8_unsuffixed(10));

preservation!(it_preserves_f64_suffixed   : FloatLit :   f64_suffixed(1.1));
preservation!(it_preserves_f64_unsuffixed : FloatLit : f64_unsuffixed(1.1));

preservation!(it_preserves_strings     : StringLit : string("hello world"));
preservation!(it_preserves_raw_strings : StringLit : string(r#"hello world"#));

preservation!(it_preserves_char : CharLit : character('1'));

preservation!(it_preserves_bytestring : ByteStringLit : byte_string(b"hello world"));
