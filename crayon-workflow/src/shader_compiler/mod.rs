//! Shader compiler
//!
//! When developing game, there is a realistic question that existing shader methologies can
//! create combinatorial explosions of shaders and can make your shader implementation become very
//! restricted and interdependent. This can even happens for same shader language in different
//! versions and targets.
//!
//! There are several approaches at handling the problem, and several solutions in that space, at
//! varying levels of completeness. I decide to go with a custom shader language for personal
//! interests :-).
//!
//! This module is the compiler for crayon's custom shader. It's able to parse formatted source into
//! an abstract syntax tree (AST). That AST can then be transformed into optimized GLSL with
//! additional informations which could be used to create pipeline state object (PSO) at runtime.

#[macro_use]
pub mod lex;
pub mod syntax;
pub mod backend;

pub use self::lex::*;
pub use self::syntax::{Program, Expression, Statement, FunctionStatement};

pub fn parse(bytes: &[u8]) -> Program {
    let tokens = tokenize(&bytes).to_result().unwrap();
    println!("{:?}", tokens);
    syntax::parse(Tokens::new(&tokens)).to_result().unwrap()
}