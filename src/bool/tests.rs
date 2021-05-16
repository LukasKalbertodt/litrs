use crate::{
    Lit, Bool,
    test_util::assert_parse_ok_eq,
};

macro_rules! assert_bool_parse {
    ($input:literal, $expected:expr) => {
        assert_parse_ok_eq($input, Lit::parse($input), Lit::Bool($expected), "Lit::parse");
        assert_parse_ok_eq($input, Bool::parse($input), $expected, "Bool::parse");
    };
}



#[test]
fn ok() {
    assert_bool_parse!("false", Bool::False);
    assert_bool_parse!("true", Bool::True);
}

#[test]
fn err() {
    assert!(Lit::parse("fa").is_err());
    assert!(Lit::parse("fal").is_err());
    assert!(Lit::parse("fals").is_err());
    assert!(Lit::parse(" false").is_err());
    assert!(Lit::parse("false ").is_err());
    assert!(Lit::parse("False").is_err());

    assert!(Lit::parse("tr").is_err());
    assert!(Lit::parse("tru").is_err());
    assert!(Lit::parse(" true").is_err());
    assert!(Lit::parse("true ").is_err());
    assert!(Lit::parse("True").is_err());
}
