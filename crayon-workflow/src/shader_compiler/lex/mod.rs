pub mod lit;
pub mod ops;
pub mod punctuation;
pub mod ident;

pub use self::lit::Literial;
pub use self::ops::Operator;
pub use self::punctuation::Punctuation;
pub use self::ident::{Ident, Type, Qualifier};

use nom;
use std::ops::{Range, RangeTo, RangeFrom, RangeFull};
use std::iter::Enumerate;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literial(Literial),
    Ident(Ident),
    Operator(Operator),
    Punctuation(Punctuation),
    Illegal,
    EOF,
}

/// Tokenize bytes into tokens.
pub fn tokenize(bytes: &[u8]) -> nom::IResult<&[u8], Vec<Token>> {
    lex_tokens(bytes).map(|v| [&v[..], &vec![Token::EOF][..]].concat())
}

use self::punctuation::parse as lex_punctuation;
use self::ops::parse as lex_operator;
use self::lit::parse as lex_literal;
use self::ident::parse as lex_ident;

named!(lex_token<&[u8], Token>, alt_complete!(
    map!(lex_operator, from_op) |
    map!(lex_punctuation, from_punctuation) |
    map!(lex_literal, from_lit) |
    map!(lex_ident, from_ident) |
    lex_illegal
));

named!(lex_tokens<&[u8], Vec<Token>>, ws!(many0!(lex_token)));

named!(lex_illegal<&[u8], Token>,
    do_parse!(take!(1) >> (Token::Illegal))
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens<'a> {
    pub tokens: &'a [Token],
    pub start: usize,
    pub end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(vec: &'a Vec<Token>) -> Self {
        Tokens {
            tokens: vec.as_slice(),
            start: 0,
            end: vec.len(),
        }
    }
}

impl<'a> nom::InputLength for Tokens<'a> {
    #[inline]
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl nom::InputLength for Token {
    #[inline]
    fn input_len(&self) -> usize {
        1
    }
}

impl<'a> nom::Slice<Range<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: Range<usize>) -> Self {
        Tokens {
            tokens: self.tokens.slice(range.clone()),
            start: self.start + range.start,
            end: self.start + range.end,
        }
    }
}

impl<'a> nom::Slice<RangeTo<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> nom::Slice<RangeFrom<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> nom::Slice<RangeFull> for Tokens<'a> {
    #[inline]
    fn slice(&self, _: RangeFull) -> Self {
        Tokens {
            tokens: self.tokens,
            start: self.start,
            end: self.end,
        }
    }
}

impl<'a> nom::InputIter for Tokens<'a> {
    type Item = &'a Token;
    type RawItem = Token;
    type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
    type IterElem = ::std::slice::Iter<'a, Token>;

    #[inline]
    fn iter_indices(&self) -> Enumerate<::std::slice::Iter<'a, Token>> {
        self.tokens.iter().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> ::std::slice::Iter<'a, Token> {
        self.tokens.iter()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
        where P: Fn(Self::RawItem) -> bool
    {
        self.tokens.iter().position(|b| predicate(b.clone()))
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Option<usize> {
        if self.tokens.len() >= count {
            Some(count)
        } else {
            None
        }
    }
}

fn from_op(op: Operator) -> Token {
    Token::Operator(op)
}

fn from_ident(ident: Ident) -> Token {
    Token::Ident(ident)
}

fn from_punctuation(pun: Punctuation) -> Token {
    Token::Punctuation(pun)
}

fn from_lit(lit: Literial) -> Token {
    Token::Literial(lit)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_case() {
        let input = "int five = 5; ".as_bytes();

        let result = tokenize(input).to_result().unwrap();
        let expected = vec![Token::Ident(Ident::Type(Type::Int)),
                            Token::Ident(Ident::Str("five".to_owned())),
                            Token::Operator(Operator::Assign),
                            Token::Literial(Literial::Int(5)),
                            Token::Punctuation(Punctuation::SemiColon),
                            Token::EOF];

        assert_eq!(result, expected);
    }

    #[test]
    fn lexer_case_2() {
        let input = "int result = func(five, ten); ".as_bytes();

        let result = tokenize(input).to_result().unwrap();
        let expected = vec![Token::Ident(Ident::Type(Type::Int)),
                            Token::Ident(Ident::Str("result".to_owned())),
                            Token::Operator(Operator::Assign),
                            Token::Ident(Ident::Str("func".to_owned())),
                            Token::Punctuation(Punctuation::LParen),
                            Token::Ident(Ident::Str("five".to_owned())),
                            Token::Punctuation(Punctuation::Comma),
                            Token::Ident(Ident::Str("ten".to_owned())),
                            Token::Punctuation(Punctuation::RParen),
                            Token::Punctuation(Punctuation::SemiColon),
                            Token::EOF];

        assert_eq!(result, expected);
    }

    #[test]
    fn lexer_case_3() {
        let input = "if (a == 10) {\n return a;\n }\n".as_bytes();
        let result = tokenize(input).to_result().unwrap();
        let expected = vec![Token::Ident(Ident::If),
                            Token::Punctuation(Punctuation::LParen),
                            Token::Ident(Ident::Str("a".to_owned())),
                            Token::Operator(Operator::BoolEq),
                            Token::Literial(Literial::Int(10)),
                            Token::Punctuation(Punctuation::RParen),
                            Token::Punctuation(Punctuation::LBrace),
                            Token::Ident(Ident::Return),
                            Token::Ident(Ident::Str("a".to_owned())),
                            Token::Punctuation(Punctuation::SemiColon),
                            Token::Punctuation(Punctuation::RBrace),
                            Token::EOF];
        assert_eq!(result, expected);
    }
}