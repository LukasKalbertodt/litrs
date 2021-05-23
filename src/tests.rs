use crate::Literal;

#[test]
fn empty() {
    assert_err!(Literal, "", Empty, None);
}
