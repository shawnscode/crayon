use super::super::lex::*;
use super::expr::*;
use crayon::graphics::pipeline;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    MetadataBind(Metadata),
    PriorVariable(PriorVariable),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Metadata {
    VertexShader(String),
    FragmentShader(String),
    DepthTest(pipeline::Comparison),
    DepthWrite(bool),
    Blend((pipeline::Equation, pipeline::BlendFactor, pipeline::BlendFactor)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorVariable {
    pub qualifier: Qualifier,
    pub tt: Type,
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub ident: String,
    pub ret: Type,
    pub params: Vec<(Type, String)>,
    pub codes: Vec<FunctionStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionStatement {
    VariableBind(VariableBind),
    If(If),
    Expression(Expression),
    Return(Return),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableBind {
    pub tt: Option<Type>,
    pub ident: String,
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub cond: Box<Expression>,
    pub consequence: Vec<FunctionStatement>,
    pub alternative: Option<Vec<FunctionStatement>>,
}

named!(pub parse_statement<Tokens, Statement>, alt_complete!(
    map!(parse_metadata, Statement::MetadataBind) |
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
    (Function { ret: ret, ident: ident, params: params, codes: codes })
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

named!(pub parse_metadata<Tokens, Metadata>, alt_complete!(
    parse_metadata_vs |
    parse_metadata_fs |
    parse_metadata_depth_write |
    parse_metadata_depth_test |
    parse_metadata_blend
));

named!(parse_metadata_vs<Tokens, Metadata>, do_parse!(
    tag_token!(Token::Ident(Ident::Metadata)) >>
    tag_token_str_case_insensitive!("VertexShader") >>
    ident: parse_str_ident >>
    (Metadata::VertexShader(ident))
));

named!(parse_metadata_fs<Tokens, Metadata>, do_parse!(
    tag_token!(Token::Ident(Ident::Metadata)) >>
    tag_token_str_case_insensitive!("FragmentShader") >>
    ident: parse_str_ident >>
    (Metadata::FragmentShader(ident))
));

named!(parse_metadata_depth_write<Tokens, Metadata>, do_parse!(
    tag_token!(Token::Ident(Ident::Metadata)) >>
    tag_token_str_case_insensitive!("DepthWrite") >>
    enable: parse_metadata_bool >>
    (Metadata::DepthWrite(enable))
));

named!(parse_metadata_depth_test<Tokens, Metadata>, do_parse!(
    tag_token!(Token::Ident(Ident::Metadata)) >>
    tag_token_str_case_insensitive!("DepthTest") >>
    cmp: parse_metadata_comparsion >>
    (Metadata::DepthTest(cmp))
));

named!(parse_metadata_blend<Tokens, Metadata>, do_parse!(
    tag_token!(Token::Ident(Ident::Metadata)) >>
    tag_token_str_case_insensitive!("Blend") >>
    eq: parse_metadata_equation >>
    f1: parse_metadata_blend_factor >>
    f2: parse_metadata_blend_factor >>
    (Metadata::Blend((eq, f1, f2)))
));

named!(parse_metadata_bool<Tokens, bool>, alt_complete!(
    value!(true, tag_token_str_case_insensitive!("true")) |
    value!(false, tag_token_str_case_insensitive!("false"))
));

named!(parse_metadata_comparsion<Tokens, pipeline::Comparison>, alt_complete! (
    value!(pipeline::Comparison::Never, tag_token_str_case_insensitive!("Never")) |
    value!(pipeline::Comparison::Less, tag_token_str_case_insensitive!("Less")) |
    value!(pipeline::Comparison::LessOrEqual, tag_token_str_case_insensitive!("LessOrEq")) |
    value!(pipeline::Comparison::Greater, tag_token_str_case_insensitive!("Greater")) |
    value!(pipeline::Comparison::GreaterOrEqual, tag_token_str_case_insensitive!("GreaterOrEq")) |
    value!(pipeline::Comparison::Equal, tag_token_str_case_insensitive!("Eq")) |
    value!(pipeline::Comparison::NotEqual, tag_token_str_case_insensitive!("Neq")) |
    value!(pipeline::Comparison::Always, tag_token_str_case_insensitive!("Always"))
));

named!(parse_metadata_equation<Tokens, pipeline::Equation>, alt_complete!(
    value!(pipeline::Equation::Add, tag_token_str_case_insensitive!("Add")) |
    value!(pipeline::Equation::Subtract, tag_token_str_case_insensitive!("Sub")) |
    value!(pipeline::Equation::ReverseSubtract, tag_token_str_case_insensitive!("ReverseSub"))
));

named!(parse_metadata_blend_factor<Tokens, pipeline::BlendFactor>, alt_complete!(
    value!(pipeline::BlendFactor::Zero, tag_token_str_case_insensitive!("Zero")) |
    value!(pipeline::BlendFactor::One, tag_token_str_case_insensitive!("One")) |
    value!(
        pipeline::BlendFactor::Value(pipeline::BlendValue::SourceAlpha),
        tag_token_str_case_insensitive!("SrcAlpha")) |
    value!(
        pipeline::BlendFactor::Value(pipeline::BlendValue::DestinationAlpha),
        tag_token_str_case_insensitive!("DstAlpha")) |
    value!(
        pipeline::BlendFactor::OneMinusValue(pipeline::BlendValue::SourceAlpha),
        tag_token_str_case_insensitive!("OneSubSrcAlpha")) |
    value!(
        pipeline::BlendFactor::OneMinusValue(pipeline::BlendValue::DestinationAlpha),
        tag_token_str_case_insensitive!("OneSubDstAlpha"))
));


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