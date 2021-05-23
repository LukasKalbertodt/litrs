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

macro_rules! assert_err {
    ($ty:ident, $input:literal, $kind:ident, $( $span:tt )+ ) => {
        assert_err_single!($ty::parse($input), $kind, $($span)+);
        assert_err_single!($crate::Lit::parse($input), $kind, $($span)+);
    };
}

macro_rules! assert_err_single {
    ($expr:expr, $kind:ident, $( $span:tt )+ ) => {
        let res = $expr;
        let err = match res {
            Err(e) => e,
            Ok(v) => panic!(
                "Expected `{}` to return an error, but it returned Ok({:?})",
                stringify!($expr),
                v,
            ),
        };
        if err.kind != $crate::ErrorKind::$kind {
            panic!(
                "Expected error kind {} for `{}` but got {:?}",
                stringify!($kind),
                stringify!($expr),
                err.kind,
            )
        }
        let expected_span = assert_err_single!(@span $($span)+);
        if err.span != expected_span {
            panic!(
                "Expected error span {:?} for `{}` but got {:?}",
                expected_span,
                stringify!($expr),
                err.span,
            )
        }
    };
    (@span $start:literal .. $end:literal) => { Some($start .. $end) };
    (@span $at:literal) => { Some($at.. $at + 1) };
    (@span None) => { None };
}
