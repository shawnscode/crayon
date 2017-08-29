
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punctuation {
    /// `,`
    Comma,
    /// `;`
    SemiColon,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `.`
    Dot,
}

named!(pub parse<Punctuation>, alt_complete!(
    value!(Punctuation::Comma, tag!(",")) |
    value!(Punctuation::SemiColon, tag!(";")) |
    value!(Punctuation::LParen, tag!("(")) |
    value!(Punctuation::RParen, tag!(")")) |
    value!(Punctuation::LBrace, tag!("{")) |
    value!(Punctuation::RBrace, tag!("}")) |
    value!(Punctuation::LBracket, tag!("[")) |
    value!(Punctuation::RBracket, tag!("]")) |
    value!(Punctuation::Dot, tag!("."))
));

#[cfg(test)]
mod tests {
    use super::{parse, Punctuation};
    use nom::*;

    #[test]
    fn puns() {
        assert_eq!(parse(&b","[..]),
                   IResult::Done(&b""[..], Punctuation::Comma));
        assert_eq!(parse(&b";"[..]),
                   IResult::Done(&b""[..], Punctuation::SemiColon));
        assert_eq!(parse(&b"("[..]),
                   IResult::Done(&b""[..], Punctuation::LParen));
        assert_eq!(parse(&b")"[..]),
                   IResult::Done(&b""[..], Punctuation::RParen));
        assert_eq!(parse(&b"{"[..]),
                   IResult::Done(&b""[..], Punctuation::LBrace));
        assert_eq!(parse(&b"}"[..]),
                   IResult::Done(&b""[..], Punctuation::RBrace));
        assert_eq!(parse(&b"["[..]),
                   IResult::Done(&b""[..], Punctuation::LBracket));
        assert_eq!(parse(&b"]"[..]),
                   IResult::Done(&b""[..], Punctuation::RBracket));
        assert_eq!(parse(&b"."[..]), IResult::Done(&b""[..], Punctuation::Dot));
    }
}