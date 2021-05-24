use crate::{ByteStringLit, CharLit, FloatLit, IntegerLit, Literal, StringLit};

#[test]
fn empty() {
    assert_err!(Literal, "", Empty, None);
}

#[test]
fn invalid_literals() {
    assert_err_single!(Literal::parse("."), InvalidLiteral, None);
    assert_err_single!(Literal::parse("+"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("-"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("e"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("e8"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("f32"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("foo"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("inf"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("nan"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("NaN"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("NAN"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("_2.7"), InvalidLiteral, None);
    assert_err_single!(Literal::parse(".5"), InvalidLiteral, None);
}

#[test]
fn misc() {
    assert_err_single!(Literal::parse("0x44.5"), InvalidIntegerTypeSuffix, 4..6);
    assert_err_single!(Literal::parse("a"), InvalidLiteral, None);
    assert_err_single!(Literal::parse(";"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("0;"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0a"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0z"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse(" 0"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("0 "), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0a3"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0z3"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("_"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("_3"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("12a3"), UnexpectedChar, 2);
    assert_err_single!(Literal::parse("12f3"), InvalidFloatTypeSuffix, 2..4);
    assert_err_single!(Literal::parse("12f_"), InvalidFloatTypeSuffix, 2..4);
    assert_err_single!(Literal::parse("12F_"), UnexpectedChar, 2);
    assert_err_single!(Literal::parse("a_123"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("B_123"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("54321a64"), UnexpectedChar, 5);
}

macro_rules! assert_no_panic {
    ($input:expr) => {
        let arr = $input;
        let input = std::str::from_utf8(&arr).expect("not unicode");
        let res = std::panic::catch_unwind(move || {
            let _ = Literal::parse(input);
            let _ = crate::BoolLit::parse(input);
            let _ = crate::IntegerLit::parse(input);
            let _ = crate::FloatLit::parse(input);
            let _ = crate::CharLit::parse(input);
            let _ = crate::StringLit::parse(input);
            let _ = crate::ByteLit::parse(input);
            let _ = crate::ByteStringLit::parse(input);
        });

        if let Err(e) = res {
            println!("\n!!! panic for: {:?}", input);
            std::panic::resume_unwind(e);
        }
    };
}

#[test]
fn never_panic_up_to_3() {
    for a in 0..128 {
        assert_no_panic!([a]);
        for b in 0..128 {
            assert_no_panic!([a, b]);
            for c in 0..128 {
                assert_no_panic!([a, b, c]);
            }
        }
    }
}

// This test takes super long in debug mode, but in release mode it's fine.
#[test]
#[ignore]
fn never_panic_len_4() {
    for a in 0..128 {
        for b in 0..128 {
            for c in 0..128 {
                for d in 0..128 {
                    assert_no_panic!([a, b, c, d]);
                }
            }
        }
    }
}

#[cfg(feature = "proc-macro2")]
#[test]
fn proc_macro() {
    assert_eq!(
        Literal::from(proc_macro2::Literal::u16_suffixed(2700)),
        Literal::Integer(IntegerLit::parse("2700u16".to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::i16_unsuffixed(3912)),
        Literal::Integer(IntegerLit::parse("3912".to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::f32_unsuffixed(3.14)),
        Literal::Float(FloatLit::parse("3.14".to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::f64_suffixed(99.3)),
        Literal::Float(FloatLit::parse("99.3f64".to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::string("hello 🦊")),
        Literal::String(StringLit::parse(r#""hello 🦊""#.to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::byte_string(b"hello \nfoxxo")),
        Literal::ByteString(ByteStringLit::parse(r#"b"hello \nfoxxo""#.to_string()).unwrap()),
    );
    assert_eq!(
        Literal::from(proc_macro2::Literal::character('🦀')),
        Literal::Char(CharLit::parse("'🦀'".to_string()).unwrap()),
    );
}
