use std::str;

use nom;
use nom::{is_alphabetic, alphanumeric};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Qualifier {
    Attribute,
    Uniform,
    Varying,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

macro_rules! check(
    ($input:expr, $submac:ident!( $($args:tt)* )) => (
        {
            let mut failed = false;
            for &idx in $input {
                if !$submac!(idx, $($args)*) {
                    failed = true;
                    break;
                }
            }
            if failed {
                nom::IResult::Error(nom::ErrorKind::Custom(20))
            } else {
                nom::IResult::Done(&b""[..], $input)
            }
        }
    );
    ($input:expr, $f:expr) => (
        check!($input, call!($f));
    );
);

fn parse_reserved(c: &str, rest: Option<&str>) -> Ident {
    let mut string = c.to_owned();
    string.push_str(rest.unwrap_or(""));

    match string.as_ref() {
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
        _ => Ident::Str(string),
    }
}

named!(take_1_char, flat_map!(take!(1), check!(is_alphabetic)));

named!(pub parse<&[u8], Ident>, do_parse!(
    c: map_res!(call!(take_1_char), str::from_utf8) >>
    rest: opt!(complete!(map_res!(alphanumeric, str::from_utf8))) >>
    (parse_reserved(c, rest))
));


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
        assert_eq!(parse(&b"3"[..]), IResult::Error(ErrorKind::Custom(20)));
        assert_eq!(parse(&b"3asd"[..]), IResult::Error(ErrorKind::Custom(20)));
    }
}