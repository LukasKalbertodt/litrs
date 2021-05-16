use crate::{Lit, Error};

#[test]
fn empty() {
    assert!(matches!(Lit::parse(""), Err(Error::Empty)));
}
