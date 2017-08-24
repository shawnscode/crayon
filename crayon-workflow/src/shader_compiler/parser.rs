use nom::*;
use super::lex::*;
use super::ast::*;

macro_rules! tag_token (
    ($i: expr, $tag: expr) => ({
        let (i1, t1) = try_parse!($i, take!(1));
        if t1.tokens.is_empty() {
            IResult::Incomplete::<_,_,u32>(Needed::Size(1))
        } else {
            if t1.tokens[0] == $tag {
                IResult::Done(i1, t1)
            } else {
                IResult::Error(error_position!(ErrorKind::Count, $i))
            }
        }
    });
);

macro_rules! take_tag_token (
    ($i: expr, $tag: expr) => ({
        let (i1, t1) = try_parse!($i, take!(1));
        if t1.tokens.is_empty() {
            IResult::Incomplete::<_,_,u32>(Needed::Size(1))
        } else {
            if t1.tokens[0] == $tag {
                IResult::Done(i1, t1.tokens[0].clone())
            } else {
                IResult::Error(error_position!(ErrorKind::Count, $i))
            }
        }
    });
);

named!(pub parse<Tokens, Program>,
    do_parse!(
        statements: many0!(parse_statement) >>
        tag_token!(Token::EOF) >>
        (statements)
    )
);

named!(pub parse_statement<Tokens, Statement>, alt_complete!(
    map!(parse_prior_variable, Statement::PriorVariable) |
    map!(parse_function, Statement::Function)
));

/// All the variables that a shader is going to use must be declared prior to use.
/// It consists of a qualifier, a build-in type and a user-defined identifier.
///
/// # Example
///
/// ```
/// attribute vec4 Position;
/// uniform mat4 u_ProjectViewMatrix;
/// ```
named!(pub parse_prior_variable<Tokens, PriorVariable>, do_parse!(
    qualifier: parse_qualifier_ident >>
    tt: parse_type_ident >>
    ident: parse_str_ident >>
    tag_token!(Token::Punctuation(Punctuation::SemiColon)) >>
    (PriorVariable {qualifier, tt, ident})
));

///
named!(pub parse_function<Tokens, Function>, do_parse!(
    ret: parse_type_ident >>
    ident: parse_str_ident >>
    tag_token!(Token::Punctuation(Punctuation::LParen)) >>
    params: parse_function_params >>
    tag_token!(Token::Punctuation(Punctuation::RParen)) >>
    codes: parse_function_statements >>
    (Function { ret: ret, ident: ident, params: params, codes: Vec::new() })
));

named!(parse_function_params<Tokens, Vec<(Type, String)>>, map!(
    opt!(do_parse!(
        p1t: parse_type_ident >>
        p1: parse_str_ident >>
        params: many0!(do_parse!(
            tag_token!(Token::Punctuation(Punctuation::Comma)) >>
            tt: parse_type_ident >>
            ident: parse_str_ident >>
            ((tt, ident))
        )) >>
        ([&vec!((p1t, p1))[..], &params[..]].concat())
    )),
    |v| v.unwrap_or(Vec::new())
));

named!(parse_function_statements<Tokens, Vec<FunctionStatement>>, do_parse!(
    tag_token!(Token::Punctuation(Punctuation::LBrace)) >>
    ss: many0!(parse_function_statement) >>
    tag_token!(Token::Punctuation(Punctuation::RBrace)) >>
    (ss)
));

named!(parse_function_statement<Tokens, FunctionStatement>, alt_complete!(
    map!(parse_variable_bind, FunctionStatement::VariableBind) |
    map!(parse_return, FunctionStatement::Return) |
    map!(parse_if, FunctionStatement::If) |
    do_parse!(
        expr: parse_expr >>
        tag_token!(Token::Punctuation(Punctuation::SemiColon)) >>
        (FunctionStatement::Expression(expr))
    )
));

named!(parse_variable_bind<Tokens, VariableBind>, do_parse!(
    tt: opt!(parse_type_ident) >>
    ident: parse_str_ident >>
    tag_token!(Token::Operator(Operator::Assign)) >>
    expr: parse_expr >>
    tag_token!(Token::Punctuation(Punctuation::SemiColon)) >>
    (VariableBind {tt: tt, ident: ident, expr: expr})
));

named!(parse_return<Tokens, Return>, do_parse!(
    tag_token!(Token::Ident(Ident::Return)) >>
    expr: parse_expr >>
    tag_token!(Token::Punctuation(Punctuation::SemiColon)) >>
    (Return { expr: expr })
));

named!(parse_if<Tokens, If>, do_parse!(
    tag_token!(Token::Ident(Ident::If)) >>
    tag_token!(Token::Punctuation(Punctuation::LParen)) >>
    expr: parse_expr >>
    tag_token!(Token::Punctuation(Punctuation::RParen)) >>
    c: parse_function_statements >>
    a: opt!(do_parse!(
        tag_token!(Token::Ident(Ident::Else)) >>
        b: parse_function_statements >>
        (b)
    )) >>
    (If { cond: Box::new(expr), consequence: c, alternative: a })
));

named!(parse_expr<Tokens, Expression>, apply!(parse_expr_from, Precedence::Lowest));

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

macro_rules! parse_ident (
    ($i: expr,) => ({
        let (i1, t1) = try_parse!($i, take!(1));
        if t1.tokens.is_empty() {
            IResult::Error(error_position!(ErrorKind::Tag, $i))
        } else {
            match t1.tokens[0].clone() {
                Token::Ident(v) => IResult::Done(i1, v),
                _ => IResult::Error(error_position!(ErrorKind::Tag, $i)),
            }
        }
    });
);

named!(parse_type_ident<Tokens, Type>,
    map_opt!(parse_ident!(), |v| {
        match v {
            Ident::Type(tt) => Some(tt),
            _ => None,
        }
    })
);

named!(parse_qualifier_ident<Tokens, Qualifier>,
    map_opt!(parse_ident!(), |v| {
        match v {
            Ident::Qualifier(qq) => Some(qq),
            _ => None,
        }
    })
);

named!(parse_str_ident<Tokens, String>,
    map_opt!(parse_ident!(), |v| {
        match v {
            Ident::Str(ss) => Some(ss),
            _ => None,
        }
    })
);

#[cfg(test)]
mod tests {
    use super::*;

    fn test_prior_variable(src: &str, expected: PriorVariable) {
        let m = tokenize(src.as_bytes()).to_result().unwrap();
        let tokens = Tokens::new(&m);

        let v = parse_prior_variable(tokens).to_result().unwrap();
        assert_eq!(v, expected);
    }

    fn empty_pv() -> PriorVariable {
        PriorVariable {
            qualifier: Qualifier::Attribute,
            tt: Type::Int,
            ident: "".to_owned(),
        }
    }

    #[test]
    fn prior_variable() {
        test_prior_variable("attribute vec2 Position;",
                            PriorVariable {
                                qualifier: Qualifier::Attribute,
                                tt: Type::Vec2,
                                ident: "Position".to_owned(),
                            });

        test_prior_variable("uniform int Position00;",
                            PriorVariable {
                                qualifier: Qualifier::Uniform,
                                tt: Type::Int,
                                ident: "Position00".to_owned(),
                            });
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable() {
        test_prior_variable("attribute vec2 000Position;", empty_pv());
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable_2() {
        test_prior_variable("vec2 Position;", empty_pv());
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable_3() {
        test_prior_variable("uniform vv Position;", empty_pv());
    }

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

    fn test_func(src: &[u8], expected: Function) {
        let m = tokenize(src).to_result().unwrap();
        let tokens = Tokens::new(&m);

        let v = parse_function(tokens).to_result().unwrap();
        assert_eq!(v, expected);
    }

    #[test]
    fn function() {
        test_func(&b"void f1() {}"[..],
                  Function {
                      ident: "f1".to_owned(),
                      ret: Type::Void,
                      params: Vec::new(),
                      codes: Vec::new(),
                  });

        test_func(&b"vec4 f2(vec2 position) {}"[..],
                  Function {
                      ident: "f2".to_owned(),
                      ret: Type::Vec4,
                      params: vec![(Type::Vec2, "position".to_owned())],
                      codes: Vec::new(),
                  });

        test_func(&b"int f3(vec2 position, vec2 texcoord) {}"[..],
                  Function {
                      ident: "f3".to_owned(),
                      ret: Type::Int,
                      params: vec![(Type::Vec2, "position".to_owned()),
                                   (Type::Vec2, "texcoord".to_owned())],
                      codes: Vec::new(),
                  });
    }
}