use nom::*;
use super::super::lex::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Identifier expression.
    Ident(String),
    /// Literal expressions.
    Literial(Literial),
    /// A unary expression, gathering a single expression and a unary operator.
    Unary(UnaryOperator, Box<Expression>),
    /// A binary expression, gathering two expressions and a binary operator.
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    /// A functional call. It has a function identifier and a list of expressions (arguments).
    Call(Box<Expression>, Vec<Expression>),
    /// Index
    Index(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,
    Eq,
    Comparations,
    Sum,
    Product,
    Call,
    Index,
}

/// All the unary opartors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// `++ expr`
    Inc,
    /// `-- expr`
    Dec,
    /// `+ expr`
    Add,
    /// `- expr`
    Minus,
    /// `! expr`
    Not,
    /// `~ expr`
    Complement,
}

/// All the binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// ` expr1 || expr2 `
    Or,
    /// ` expr1 && expr2 `
    And,
    /// ` expr1 | expr2 `
    BitOr,
    /// ` expr1 ^ expr2 `
    BitXor,
    /// ` expr1 & expr2 `
    BitAnd,
    /// ` expr1 == expr2 `
    Eq,
    /// ` expr1 != expr2 `
    Neq,
    /// ` expr1 < expr2`
    LT,
    /// ` expr1 > expr2`
    GT,
    /// ` expr1 <= expr2`
    LTE,
    /// ` expr1 >= expr2`
    GTE,
    /// ` expr1 << expr2`
    LShift,
    /// ` expr1 >> expr2`
    RShift,
    /// ` expr1 + expr2`
    Add,
    /// ` expr1 - expr2`
    Minus,
    /// ` expr1 * expr2`
    Mul,
    /// ` expr1 / expr2`
    Div,
    /// ` expr1 % expr2`
    Mod,
}

named!(pub parse_expr<Tokens, Expression>, apply!(parse_expr_from, Precedence::Lowest));

fn parse_expr_from(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expression> {
    do_parse!(input,
        head: parse_atomic_expr >>
        i: apply!(parse_expr_recursive, precedence, head) >>
        (i)
    )
}

fn parse_expr_recursive(input: Tokens,
                        precedence: Precedence,
                        head: Expression)
                        -> IResult<Tokens, Expression> {
    let (i1, t1) = try_parse!(input, take!(1));
    if t1.tokens.is_empty() {
        IResult::Done(i1, head)
    } else {
        let next = t1.tokens[0].clone();
        match parse_next_expr_token(&next) {
            (Precedence::Call, _) if precedence < Precedence::Call => {
                let (i2, head2) = try_parse!(input, apply!(parse_call_expr, head));
                parse_expr_recursive(i2, precedence, head2)
            }
            (Precedence::Index, _) if precedence < Precedence::Index => {
                let (i2, head2) = try_parse!(input, apply!(parse_index_expr, head));
                parse_expr_recursive(i2, precedence, head2)
            }
            (ref prece, _) if precedence < *prece => {
                let (i2, head2) = try_parse!(input, apply!(parse_binary_expr, head));
                parse_expr_recursive(i2, precedence, head2)
            }
            _ => IResult::Done(input, head),
        }
    }
}

fn parse_call_expr(input: Tokens, ident: Expression) -> IResult<Tokens, Expression> {
    do_parse!(input,
              tag_token!(Token::Punctuation(Punctuation::LParen)) >> args: parse_call_params >>
              tag_token!(Token::Punctuation(Punctuation::RParen)) >>
              (Expression::Call(Box::new(ident), args)))
}

named!(parse_call_params<Tokens, Vec<Expression>>, map!(
    opt!(do_parse!(
        p: parse_expr >>
        params: many0!(do_parse!(
            tag_token!(Token::Punctuation(Punctuation::Comma)) >>
            pp: parse_expr >>
            (pp)
        )) >>
        ([&vec!(p)[..], &params[..]].concat())
    )),
    |v| v.unwrap_or(Vec::new())
));

fn parse_index_expr(input: Tokens, ident: Expression) -> IResult<Tokens, Expression> {
    do_parse!(input,
              tag_token!(Token::Punctuation(Punctuation::LBracket)) >> index: parse_expr >>
              tag_token!(Token::Punctuation(Punctuation::RBracket)) >>
              (Expression::Index(Box::new(ident), Box::new(index))))
}

fn parse_binary_expr(input: Tokens, lhs: Expression) -> IResult<Tokens, Expression> {
    let (i1, t1) = try_parse!(input, take!(1));
    if t1.tokens.is_empty() {
        IResult::Error(error_position!(ErrorKind::Tag, input))
    } else {
        let next = t1.tokens[0].clone();
        let (precedence, maybe_op) = parse_next_expr_token(&next);
        match maybe_op {
            None => IResult::Error(error_position!(ErrorKind::Tag, input)),
            Some(op) => {
                let (i2, rhs) = try_parse!(i1, apply!(parse_expr_from, precedence));
                IResult::Done(i2, Expression::Binary(op, Box::new(lhs), Box::new(rhs)))
            }
        }
    }
}

fn parse_next_expr_token(token: &Token) -> (Precedence, Option<BinaryOperator>) {
    match *token {
        Token::Operator(v) => {
            match v {
                Operator::BoolEq => (Precedence::Eq, Some(BinaryOperator::Eq)),
                Operator::BoolNeq => (Precedence::Eq, Some(BinaryOperator::Neq)),
                Operator::BoolLessThan => (Precedence::Comparations, Some(BinaryOperator::LT)),
                Operator::BoolGreaterThan => (Precedence::Comparations, Some(BinaryOperator::GT)),
                Operator::BoolLessThanEqual => {
                    (Precedence::Comparations, Some(BinaryOperator::LTE))
                }
                Operator::BoolGreaterThanEqual => {
                    (Precedence::Comparations, Some(BinaryOperator::GTE))
                }
                Operator::Add => (Precedence::Sum, Some(BinaryOperator::Add)),
                Operator::Minus => (Precedence::Sum, Some(BinaryOperator::Minus)),
                Operator::Mul => (Precedence::Product, Some(BinaryOperator::Mul)),
                Operator::Div => (Precedence::Product, Some(BinaryOperator::Div)),
                _ => (Precedence::Lowest, None),
            }
        }
        Token::Punctuation(Punctuation::LParen) => (Precedence::Call, None),
        Token::Punctuation(Punctuation::LBracket) => (Precedence::Index, None),
        _ => (Precedence::Lowest, None),
    }
}

named!(parse_atomic_expr<Tokens, Expression>, alt_complete!(
    parse_lit_expr |
    map!(parse_str_ident, Expression::Ident) |
    parse_unary_expr |
    parse_paren_expr
));

macro_rules! parse_literal (
    ($i: expr,) => ({
        let (i1, t1) = try_parse!($i, take!(1));
        if t1.tokens.is_empty() {
            IResult::Error(error_position!(ErrorKind::Tag, $i))
        } else {
            match t1.tokens[0].clone() {
                Token::Literial(Literial::Int(i)) => IResult::Done(i1, Literial::Int(i)),
                Token::Literial(Literial::Float(f)) => IResult::Done(i1, Literial::Float(f)),
                _ => IResult::Error(error_position!(ErrorKind::Tag, $i)),
            }
        }
    });
);

named!(parse_lit_expr<Tokens, Expression>, do_parse!(
    lit: parse_literal!() >>
    (Expression::Literial(lit))
));

named!(parse_unary_op<Tokens, Token>, alt_complete!(
    take_tag_token!(Token::Operator(Operator::Inc)) |
    take_tag_token!(Token::Operator(Operator::Dec)) |
    take_tag_token!(Token::Operator(Operator::Add)) |
    take_tag_token!(Token::Operator(Operator::Minus)) |
    take_tag_token!(Token::Operator(Operator::BoolNot)) |
    take_tag_token!(Token::Operator(Operator::BitComplement))
));

fn parse_unary_expr(input: Tokens) -> IResult<Tokens, Expression> {
    let (i1, t1) = try_parse!(input, parse_unary_op);
    let (i2, e) = try_parse!(i1, parse_atomic_expr);

    let e = Box::new(e);
    let expr = match t1.clone() {
        Token::Operator(Operator::Inc) => Expression::Unary(UnaryOperator::Inc, e),
        Token::Operator(Operator::Dec) => Expression::Unary(UnaryOperator::Dec, e),
        Token::Operator(Operator::Add) => Expression::Unary(UnaryOperator::Add, e),
        Token::Operator(Operator::Minus) => Expression::Unary(UnaryOperator::Minus, e),
        Token::Operator(Operator::BoolNot) => Expression::Unary(UnaryOperator::Not, e),
        Token::Operator(Operator::BitComplement) => Expression::Unary(UnaryOperator::Complement, e),
        _ => return IResult::Error(ErrorKind::Custom(66)),
    };

    IResult::Done(i2, expr)
}

named!(parse_paren_expr<Tokens, Expression>, do_parse!(
    tag_token!(Token::Punctuation(Punctuation::LParen)) >>
    expr: parse_expr >>
    tag_token!(Token::Punctuation(Punctuation::RParen)) >>
    (expr)
));

#[cfg(test)]
mod tests {
    use super::*;

    fn test_expr(src: &str, expected: Expression) {
        let m = tokenize(src.as_bytes()).to_result().unwrap();
        let tokens = Tokens::new(&m);

        let v = parse_expr(tokens).to_result().unwrap();
        assert_eq!(v, expected);
    }

    #[test]
    fn expr() {
        test_expr("asdasd", Expression::Ident("asdasd".to_owned()));
        test_expr("123123", Expression::Literial(Literial::Int(123123)));
        test_expr("123123.123",
                  Expression::Literial(Literial::Float(123123.123)));
        test_expr("+2",
                  Expression::Unary(UnaryOperator::Add,
                                    Box::new(Expression::Literial(Literial::Int(2)))));
        test_expr("-1.2",
                  Expression::Unary(UnaryOperator::Minus,
                                    Box::new(Expression::Literial(Literial::Float(1.2)))));
        // test_expr("-(1.2)",
        //           Expression::Unary(UnaryOperator::Minus,
        //                             Box::new(Expression::Literial(Literial::Float(1.2)))));
    }
}