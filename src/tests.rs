use crate::Literal;

#[test]
fn empty() {
    assert_err!(Literal, "", Empty, None);
}

#[test]
fn invalid_literals() {
    assert_err_single!(Literal::parse("."), InvalidLiteral, None);
    assert_err_single!(Literal::parse("+"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("-"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("e"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("e8"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("f32"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("foo"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("inf"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("nan"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("NaN"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("NAN"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("_2.7"), InvalidLiteral, None);
    assert_err_single!(Literal::parse(".5"), InvalidLiteral, None);
}

#[test]
fn misc() {
    assert_err_single!(Literal::parse("0x44.5"), InvalidIntegerTypeSuffix, 4..6);
    assert_err_single!(Literal::parse("a"), InvalidLiteral, None);
    assert_err_single!(Literal::parse(";"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("0;"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0a"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0z"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse(" 0"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("0 "), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0a3"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("0z3"), UnexpectedChar, 1);
    assert_err_single!(Literal::parse("_"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("_3"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("12a3"), UnexpectedChar, 2);
    assert_err_single!(Literal::parse("12f3"), InvalidFloatTypeSuffix, 2..4);
    assert_err_single!(Literal::parse("12f_"), InvalidFloatTypeSuffix, 2..4);
    assert_err_single!(Literal::parse("12F_"), UnexpectedChar, 2);
    assert_err_single!(Literal::parse("a_123"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("B_123"), InvalidLiteral, None);
    assert_err_single!(Literal::parse("54321a64"), UnexpectedChar, 5);
}
