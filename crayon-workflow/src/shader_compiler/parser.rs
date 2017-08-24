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
    qualifier: map_opt!(parse_ident!(), try_extract_qualifier_ident) >>
    tt: map_opt!(parse_ident!(), try_extract_type_ident) >>
    ident: map_opt!(parse_ident!(), try_extract_str_ident) >>
    tag_token!(Token::Punctuation(Punctuation::SemiColon)) >>
    (PriorVariable {qualifier, tt, ident})
));

named!(pub parse_function<Tokens, Statement>, do_parse!(
    ret: map_opt!(parse_ident!(), try_extract_type_ident) >>
))

fn try_extract_type_ident(v: Ident) -> Option<Type> {
    match v {
        Ident::Type(tt) => Some(tt),
        _ => None,
    }
}

fn try_extract_qualifier_ident(v: Ident) -> Option<Qualifier> {
    match v {
        Ident::Qualifier(qq) => Some(qq),
        _ => None,
    }
}

fn try_extract_str_ident(v: Ident) -> Option<String> {
    match v {
        Ident::Str(ss) => Some(ss),
        _ => None,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_prior_variable(src: &[u8], expected: PriorVariable) {
        let m = tokenize(src).to_result().unwrap();
        let tokens = Tokens::new(&m);

        let v = parse_prior_variable(tokens).to_result().unwrap();
        assert_eq!(v.qualifier, expected.qualifier);
        assert_eq!(v.tt, expected.tt);
        assert_eq!(v.ident, expected.ident);
    }

    fn empty() -> PriorVariable {
        PriorVariable {
            qualifier: Qualifier::Attribute,
            tt: Type::Int,
            ident: "".to_owned(),
        }
    }

    #[test]
    fn prior_variable() {
        test_prior_variable(&b"attribute vec2 Position;"[..],
                            PriorVariable {
                                qualifier: Qualifier::Attribute,
                                tt: Type::Vec2,
                                ident: "Position".to_owned(),
                            });

        test_prior_variable(&b"uniform int Position00;"[..],
                            PriorVariable {
                                qualifier: Qualifier::Uniform,
                                tt: Type::Int,
                                ident: "Position00".to_owned(),
                            });
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable() {
        test_prior_variable(&b"attribute vec2 000Position;"[..], empty());
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable_2() {
        test_prior_variable(&b"vec2 Position;"[..], empty());
    }

    #[test]
    #[should_panic]
    fn bad_prior_variable_3() {
        test_prior_variable(&b"uniform vv Position;"[..], empty());
    }
}