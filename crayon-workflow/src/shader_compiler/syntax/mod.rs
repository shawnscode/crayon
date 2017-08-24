pub mod expr;
pub mod statement;

pub use self::expr::Expression;
pub use self::statement::{Statement, FunctionStatement};

use super::lex::{Tokens, Token};
use self::statement::parse_statement;

pub type Program = Vec<Statement>;

named!(pub parse<Tokens, Program>,
    do_parse!(
        statements: many0!(parse_statement) >>
        tag_token!(Token::EOF) >>
        (statements)
    )
);