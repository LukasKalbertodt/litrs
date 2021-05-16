use crate::*;

#[track_caller]
pub(crate) fn assert_parse_ok_eq<T: PartialEq + std::fmt::Debug>(
    input: &str,
    result: Result<T, Error>,
    expected: T,
    parse_method: &str,
) {
    match result {
        Ok(actual) if actual == expected => {}
        Ok(actual) => {
            panic!(
                "unexpected parsing result (with `{}`) for `{}`:\nactual:    {:?}\nexpected:  {:?}",
                parse_method,
                input,
                actual,
                expected,
            )
        }
        Err(e) => {
            panic!(
                "expected `{}` to be parsed (with `{}`) successfully, but it failed: {:?}",
                input,
                parse_method,
                e,
            );
        }
    }
}
