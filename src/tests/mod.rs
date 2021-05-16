use super::*;

macro_rules! assert_parse {
    ($s:literal, $expected:expr) => {{
        let result = $crate::Lit::parse($s);
        match result {
            Ok(actual) if actual == $expected => {}
            Ok(actual) => {
                panic!(
                    "unexpected parsing result for `{}`:\nactual:    {:?}\nexpected:  {:?}",
                    $s,
                    actual,
                    $expected,
                )
            }
            Err(e) => {
                panic!("expected `{}` to be parsed successfully, but it failed: {:?}", $s, e);
            }
        }
    }};
}


#[test]
fn empty() {
    assert!(matches!(Lit::parse(""), Err(Error::Empty)));
}

#[test]
fn bool_ok() {
    assert_parse!("false", Lit::Bool(Bool::False));
    assert_parse!("true", Lit::Bool(Bool::True));
}

#[test]
fn bool_err() {
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
