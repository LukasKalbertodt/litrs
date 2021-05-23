use crate::Lit;

#[test]
fn empty() {
    assert_err!(Lit::parse(""), Empty, None);
}
