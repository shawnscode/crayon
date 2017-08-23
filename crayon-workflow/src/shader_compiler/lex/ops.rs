#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `=`
    Assign,

    /// `++`
    Inc,
    /// `--`
    Dec,
    /// `+`
    Add,
    /// `-`
    Minus,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,

    /// `!`
    BoolNot,
    /// `||`
    BoolOr,
    /// `&&`
    BoolAnd,
    /// `==`
    BoolEq,
    /// `!=`
    BoolNeq,
    /// `<`
    BoolLessThan,
    /// `>`
    BoolGreaterThan,
    /// `<=`
    BoolLessThanEqual,
    /// `>=`
    BoolGreaterThanEqual,

    /// `|`
    BitOr,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,
    /// `~`
    BitComplement,
    /// `<<`
    BitLShift,
    /// `>>`
    BitRShift,
}

// These are ordered such that strings that if tag X is a prefix of tag Y, then X lies below Y.
// This ensures that something like 'a >= b' doesn't get an intermediate parsing of 'a >' and then
// fail to parse '= b'.
named!(pub parse<Operator>, alt_complete!(
    value!(Operator::Inc, tag!("++")) |
    value!(Operator::Add, tag!("+")) |
    value!(Operator::Dec, tag!("--")) |
    value!(Operator::Minus, tag!("-")) |
    value!(Operator::Mul, tag!("*")) |
    value!(Operator::Div, tag!("/")) |
    value!(Operator::Mod, tag!("%")) |

    value!(Operator::BoolEq, tag!("==")) |
    value!(Operator::BoolNeq, tag!("!=")) |

    value!(Operator::BoolAnd, tag!("&&")) |
    value!(Operator::BoolOr, tag!("||")) |
    value!(Operator::BoolNot, tag!("!")) |

    value!(Operator::BitAnd, tag!("&")) |
    value!(Operator::BitXor, tag!("^")) |
    value!(Operator::BitOr, tag!("|")) |
    value!(Operator::BitComplement, tag!("~")) |

    value!(Operator::BoolLessThanEqual, tag!("<=")) |
    value!(Operator::BoolGreaterThanEqual, tag!(">=")) |

    value!(Operator::BitLShift, tag!("<<")) |
    value!(Operator::BitRShift, tag!(">>")) |
    value!(Operator::BoolLessThan, tag!("<")) |
    value!(Operator::BoolGreaterThan, tag!(">")) |

    value!(Operator::Assign, tag!("="))
));

#[cfg(test)]
mod tests {
    use super::{parse, Operator};
    use nom::*;

    #[test]
    fn ops() {
        assert_eq!(parse(&b"++"[..]), IResult::Done(&b""[..], Operator::Inc));
        assert_eq!(parse(&b"+"[..]), IResult::Done(&b""[..], Operator::Add));
        assert_eq!(parse(&b"--"[..]), IResult::Done(&b""[..], Operator::Dec));
        assert_eq!(parse(&b"-"[..]), IResult::Done(&b""[..], Operator::Minus));
        assert_eq!(parse(&b"*"[..]), IResult::Done(&b""[..], Operator::Mul));
        assert_eq!(parse(&b"/"[..]), IResult::Done(&b""[..], Operator::Div));
        assert_eq!(parse(&b"%"[..]), IResult::Done(&b""[..], Operator::Mod));
        assert_eq!(parse(&b"!"[..]), IResult::Done(&b""[..], Operator::BoolNot));
        assert_eq!(parse(&b"||"[..]), IResult::Done(&b""[..], Operator::BoolOr));
        assert_eq!(parse(&b"&&"[..]),
                   IResult::Done(&b""[..], Operator::BoolAnd));
        assert_eq!(parse(&b"=="[..]), IResult::Done(&b""[..], Operator::BoolEq));
        assert_eq!(parse(&b"!="[..]),
                   IResult::Done(&b""[..], Operator::BoolNeq));
        assert_eq!(parse(&b"<"[..]),
                   IResult::Done(&b""[..], Operator::BoolLessThan));
        assert_eq!(parse(&b">"[..]),
                   IResult::Done(&b""[..], Operator::BoolGreaterThan));
        assert_eq!(parse(&b"<="[..]),
                   IResult::Done(&b""[..], Operator::BoolLessThanEqual));
        assert_eq!(parse(&b">="[..]),
                   IResult::Done(&b""[..], Operator::BoolGreaterThanEqual));
        assert_eq!(parse(&b"|"[..]), IResult::Done(&b""[..], Operator::BitOr));
        assert_eq!(parse(&b"^"[..]), IResult::Done(&b""[..], Operator::BitXor));
        assert_eq!(parse(&b"&"[..]), IResult::Done(&b""[..], Operator::BitAnd));
        assert_eq!(parse(&b"~"[..]),
                   IResult::Done(&b""[..], Operator::BitComplement));
        assert_eq!(parse(&b"<<"[..]),
                   IResult::Done(&b""[..], Operator::BitLShift));
        assert_eq!(parse(&b">>"[..]),
                   IResult::Done(&b""[..], Operator::BitRShift));
        assert_eq!(parse(&b"="[..]), IResult::Done(&b""[..], Operator::Assign));
    }
}