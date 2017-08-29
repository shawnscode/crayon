pub mod expr;
pub mod statement;

pub use self::expr::Expression;
pub use self::statement::{Statement, FunctionStatement, Metadata};

use super::lex::{Tokens, Token};
use self::statement::parse_statement;

use nom;

pub fn parse(tokens: Tokens) -> nom::IResult<Tokens, Vec<Statement>> {
    syntax_parse(tokens)
}

named!(syntax_parse<Tokens, Vec<Statement>>, do_parse!(
    statements: many0!(parse_statement) >>
    tag_token!(Token::EOF) >>
    (statements)
));