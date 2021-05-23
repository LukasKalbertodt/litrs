use crate::Lit;

#[test]
fn empty() {
    assert_err!(Lit, "", Empty, None);
}
