use std::str;

/// All the reserved keywords
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ident {
    /// `if`
    If,
    /// `else`
    Else,
    /// `return`
    Return,
    /// qualifiers.
    Qualifier(Qualifier),
    /// build-in types.
    Type(Type),
    /// string identifier.
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qualifier {
    Attribute,
    Uniform,
    Varying,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Void,
    Int,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
}

fn parse_reserved(ident: &str) -> Ident {
    match ident {
        "if" => Ident::If,
        "else" => Ident::Else,
        "return" => Ident::Return,
        "void" => Ident::Type(Type::Void),
        "int" => Ident::Type(Type::Int),
        "float" => Ident::Type(Type::Float),
        "vec2" => Ident::Type(Type::Vec2),
        "vec3" => Ident::Type(Type::Vec3),
        "vec4" => Ident::Type(Type::Vec4),
        "mat2" => Ident::Type(Type::Mat2),
        "mat3" => Ident::Type(Type::Mat3),
        "mat4" => Ident::Type(Type::Mat4),
        "sampler2D" => Ident::Type(Type::Sampler2D),
        "attribute" => Ident::Qualifier(Qualifier::Attribute),
        "uniform" => Ident::Qualifier(Qualifier::Uniform),
        "varying" => Ident::Qualifier(Qualifier::Varying),
        _ => Ident::Str(ident.to_owned()),
    }
}

/// Parse an identifier (raw version).
named!(identifier_str,
       do_parse!(
    name: verify!(take_while1!(identifier_pred), verify_identifier) >>
    (name)
));

/// Parse an identifier.
named!(pub parse<&[u8], Ident>, do_parse!(
    ident: map_res!(identifier_str, str::from_utf8) >>
    (parse_reserved(ident))
));

#[inline]
fn identifier_pred(c: u8) -> bool {
    let ch = char::from(c);
    ch.is_alphanumeric() || ch == '_'
}

#[inline]
fn verify_identifier(s: &[u8]) -> bool {
    !char::from(s[0]).is_digit(10)
}


#[cfg(test)]
mod tests {
    use super::*;
    use nom::*;

    #[test]
    fn ident() {
        assert_eq!(parse(&b"if"[..]), IResult::Done(&b""[..], Ident::If));
        assert_eq!(parse(&b"else"[..]), IResult::Done(&b""[..], Ident::Else));
        assert_eq!(parse(&b"return"[..]),
                   IResult::Done(&b""[..], Ident::Return));
        assert_eq!(parse(&b"void"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Void)));
        assert_eq!(parse(&b"int"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Int)));
        assert_eq!(parse(&b"float"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Float)));
        assert_eq!(parse(&b"vec2"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Vec2)));
        assert_eq!(parse(&b"vec3"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Vec3)));
        assert_eq!(parse(&b"vec4"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Vec4)));
        assert_eq!(parse(&b"mat2"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Mat2)));
        assert_eq!(parse(&b"mat3"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Mat3)));
        assert_eq!(parse(&b"mat4"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Mat4)));
        assert_eq!(parse(&b"sampler2D"[..]),
                   IResult::Done(&b""[..], Ident::Type(Type::Sampler2D)));
        assert_eq!(parse(&b"attribute"[..]),
                   IResult::Done(&b""[..], Ident::Qualifier(Qualifier::Attribute)));
        assert_eq!(parse(&b"uniform"[..]),
                   IResult::Done(&b""[..], Ident::Qualifier(Qualifier::Uniform)));
        assert_eq!(parse(&b"varying"[..]),
                   IResult::Done(&b""[..], Ident::Qualifier(Qualifier::Varying)));
        assert_eq!(parse(&b"ifv"[..]),
                   IResult::Done(&b""[..], Ident::Str("ifv".to_owned())));
        assert_eq!(parse(&b"asd123"[..]),
                   IResult::Done(&b""[..], Ident::Str("asd123".to_owned())));
        assert_eq!(parse(&b"asd123asd123"[..]),
                   IResult::Done(&b""[..], Ident::Str("asd123asd123".to_owned())));

        assert_eq!(parse(&b"3"[..]), IResult::Error(ErrorKind::Verify));
        assert_eq!(parse(&b"3asd"[..]), IResult::Error(ErrorKind::Verify));
    }
}