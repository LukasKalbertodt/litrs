use crate::{Literal, ByteStringLit, test_util::assert_parse_ok_eq};

// ===== Utility functions =======================================================================

macro_rules! check {
    ($lit:literal, $has_escapes:expr, $num_hashes:expr) => {
        let input = stringify!($lit);
        let expected = ByteStringLit {
            raw: input,
            value: if $has_escapes { Some($lit.to_vec()) } else { None },
            num_hashes: $num_hashes,
        };

        assert_parse_ok_eq(
            input, ByteStringLit::parse(input), expected.clone(), "ByteStringLit::parse");
        assert_parse_ok_eq(
            input, Literal::parse(input), Literal::ByteString(expected), "Literal::parse");
        assert_eq!(ByteStringLit::parse(input).unwrap().value(), $lit);
        // assert_eq!(ByteStringLit::parse(input).unwrap().into_value(), $lit);
    };
}


// ===== Actual tests ============================================================================

#[test]
fn simple() {
    check!(b"", false, None);
    check!(b"a", false, None);
    check!(b"peter", false, None);
}

#[test]
fn special_whitespace() {
    let strings = ["\n", "\t", "foo\tbar", "baz\n", "\r\n"];

    for &s in &strings {
        let input = format!(r#"b"{}""#, s);
        let input_raw = format!(r#"br"{}""#, s);
        for (input, num_hashes) in vec![(input, None), (input_raw, Some(0))] {
            let expected = ByteStringLit {
                raw: &*input,
                value: None,
                num_hashes,
            };
            assert_parse_ok_eq(
                &input, ByteStringLit::parse(&*input), expected.clone(), "ByteStringLit::parse");
            assert_parse_ok_eq(
                &input, Literal::parse(&*input), Literal::ByteString(expected), "Literal::parse");
            assert_eq!(ByteStringLit::parse(&*input).unwrap().value(), s.as_bytes());
            // assert_eq!(ByteStringLit::parse(&*input).unwrap().into_value(), s);
        }
    }

    let res = ByteStringLit::parse("br\"\r\"").expect("failed to parse");
    assert_eq!(res.value(), b"\r");
}

#[test]
fn simple_escapes() {
    check!(b"a\nb", true, None);
    check!(b"\nb", true, None);
    check!(b"a\n", true, None);
    check!(b"\n", true, None);

    check!(b"\x60foo \t bar\rbaz\n banana \0kiwi", true, None);
    check!(b"foo \\ferris", true, None);
    check!(b"baz \\ferris\"box", true, None);
    check!(b"\\foo\\ banana\" baz\"", true, None);
    check!(b"\"foo \\ferris \" baz\\", true, None);

    check!(b"\x00", true, None);
    check!(b" \x01", true, None);
    check!(b"\x0c foo", true, None);
    check!(b" foo\x0D ", true, None);
    check!(b"\\x13", true, None);
    check!(b"\"x30", true, None);
}

#[test]
fn raw_byte_string() {
    check!(br"", false, Some(0));
    check!(br"a", false, Some(0));
    check!(br"peter", false, Some(0));
    check!(br"Greetings jason!", false, Some(0));

    check!(br#""#, false, Some(1));
    check!(br#"a"#, false, Some(1));
    check!(br##"peter"##, false, Some(2));
    check!(br###"Greetings # Jason!"###, false, Some(3));
    check!(br########"we ## need #### more ####### hashtags"########, false, Some(8));

    check!(br#"foo " bar"#, false, Some(1));
    check!(br##"foo " bar"##, false, Some(2));
    check!(br#"foo """" '"'" bar"#, false, Some(1));
    check!(br#""foo""#, false, Some(1));
    check!(br###""foo'"###, false, Some(3));
    check!(br#""x'#_#s'"#, false, Some(1));
    check!(br"#", false, Some(0));
    check!(br"foo#", false, Some(0));
    check!(br"##bar", false, Some(0));
    check!(br###""##foo"##bar'"###, false, Some(3));

    check!(br"foo\n\t\r\0\\x60\u{123}doggo", false, Some(0));
    check!(br#"cat\n\t\r\0\\x60\u{123}doggo"#, false, Some(1));
}
