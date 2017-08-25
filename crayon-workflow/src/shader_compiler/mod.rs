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
pub mod errors;

pub use self::lex::*;
pub use self::syntax::{Expression, Statement, FunctionStatement, Metadata};

use crayon::graphics;

pub struct Shader {
    vs: Vec<Statement>,
    fs: Vec<Statement>,
    render_state: graphics::RenderState,
}

impl Shader {
    pub fn load(bytes: &[u8]) -> errors::Result<Shader> {
        let tokens = tokenize(&bytes).to_result()?;
        let statements = syntax::parse(Tokens::new(&tokens)).to_result()?;

        let mut shader = Shader {
            vs: Vec::new(),
            fs: Vec::new(),
            render_state: graphics::RenderState::default(),
        };

        let mut vs_main = "vs";
        let mut fs_main = "fs";

        for stmt in &statements {
            if let &Statement::MetadataBind(ref metadata) = stmt {
                match metadata {
                    &Metadata::VertexShader(ref v) => vs_main = &v,
                    &Metadata::FragmentShader(ref v) => fs_main = &v,
                    &Metadata::DepthTest(v) => shader.render_state.depth_test = v,
                    &Metadata::DepthWrite(v) => shader.render_state.depth_write = v,
                    &Metadata::Blend((eq, src, dst)) => {
                        shader.render_state.color_blend = Some((eq, src, dst))
                    }
                }
            }
        }

        // Parse vertex shader.
        for stmt in &statements {
            match stmt {
                &Statement::MetadataBind(_) => continue,
                &Statement::Function(ref func) => {
                    if func.ident == fs_main {
                        continue;
                    }
                }
                _ => {}
            }

            shader.vs.push(stmt.clone());
        }

        // Parse fragment shader.
        for stmt in &statements {
            match stmt {
                &Statement::MetadataBind(_) => continue,
                &Statement::Function(ref func) => {
                    if func.ident == vs_main {
                        continue;
                    }
                }
                &Statement::PriorVariable(ref pv) => {
                    if pv.qualifier == Qualifier::Attribute {
                        continue;
                    }
                }
            }

            shader.fs.push(stmt.clone());
        }

        Ok(shader)
    }

    pub fn vs(&self) -> &Vec<Statement> {
        &self.vs
    }

    pub fn fs(&self) -> &Vec<Statement> {
        &self.fs
    }

    pub fn render_state(&self) -> &graphics::RenderState {
        &self.render_state
    }
}