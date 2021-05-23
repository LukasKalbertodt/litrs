use crate::{Literal, StringLit, test_util::assert_parse_ok_eq};

// ===== Utility functions =======================================================================

macro_rules! check {
    ($lit:literal, $has_escapes:expr, $num_hashes:expr) => {
        let input = stringify!($lit);
        let expected = StringLit {
            raw: input,
            value: if $has_escapes { Some($lit.to_string()) } else { None },
            num_hashes: $num_hashes,
        };

        assert_parse_ok_eq(input, StringLit::parse(input), expected.clone(), "StringLit::parse");
        assert_parse_ok_eq(
            input, Literal::parse(input), Literal::String(expected), "Literal::parse");
        assert_eq!(StringLit::parse(input).unwrap().value(), $lit);
        assert_eq!(StringLit::parse(input).unwrap().into_value(), $lit);
    };
}


// ===== Actual tests ============================================================================


#[test]
fn raw_string() {
    check!(r"", false, Some(0));
    check!(r"a", false, Some(0));
    check!(r"peter", false, Some(0));
    check!(r"Sei gegrüßt, Bärthelt!", false, Some(0));
    check!(r"أنا لا أتحدث العربية", false, Some(0));
    check!(r"お前はもう死んでいる", false, Some(0));
    check!(r"Пушки - интересные музыкальные инструменты", false, Some(0));
    check!(r"lit 👌 😂 af", false, Some(0));

    check!(r#""#, false, Some(1));
    check!(r#"a"#, false, Some(1));
    check!(r##"peter"##, false, Some(2));
    check!(r###"Sei gegrüßt, Bärthelt!"###, false, Some(3));
    check!(r########"lit 👌 😂 af"########, false, Some(8));

    check!(r#"foo " bar"#, false, Some(1));
    check!(r##"foo " bar"##, false, Some(2));
    check!(r#"foo """" '"'" bar"#, false, Some(1));
    check!(r#""foo""#, false, Some(1));
    check!(r###""foo'"###, false, Some(3));
    check!(r#""x'#_#s'"#, false, Some(1));
    check!(r"#", false, Some(0));
    check!(r"foo#", false, Some(0));
    check!(r"##bar", false, Some(0));
    check!(r###""##foo"##bar'"###, false, Some(3));

    check!(r"さび\n\t\r\0\\x60\u{123}フェリス", false, Some(0));
    check!(r#"さび\n\t\r\0\\x60\u{123}フェリス"#, false, Some(1));
}
