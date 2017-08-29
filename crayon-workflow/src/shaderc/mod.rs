//! # Shader compiler
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
//!
//! # Syntax
//!
//! The syntax of this shader language is designed to be very similiar to GLSL for a lower learning-
//! curve.
//!
//! TODO: Specification of syntsxs

#[macro_use]
pub mod lex;
pub mod syntax;
pub mod backend;
pub mod errors;

pub use self::lex::*;
pub use self::syntax::{Expression, Statement, FunctionStatement, Metadata};
pub use self::backend::ShaderBackend;

use std::collections::HashMap;
use crayon::graphics;
use self::errors::*;

#[derive(Debug)]
pub struct Shader {
    vs_main: String,
    vs: Vec<Statement>,
    fs_main: String,
    fs: Vec<Statement>,
    uniforms: HashMap<String, graphics::UniformVariableType>,
    render_state: graphics::RenderState,
    layout: graphics::AttributeLayout,
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderPhase {
    Vertex,
    Fragment,
}

impl Shader {
    pub fn load(bytes: &[u8]) -> Result<Shader> {
        let tokens = tokenize(&bytes).to_result()?;
        let statements = syntax::parse(Tokens::new(&tokens)).to_result()?;

        let mut vs = Vec::new();
        let mut fs = Vec::new();
        let mut vs_main = "vs";
        let mut fs_main = "fs";
        let mut render_state = graphics::RenderState::default();
        let mut layout = graphics::AttributeLayoutBuilder::new();
        let mut uniforms = HashMap::new();

        for stmt in &statements {
            match stmt {
                &Statement::MetadataBind(ref metadata) => {
                    match metadata {
                        &Metadata::VertexShader(ref v) => vs_main = &v,
                        &Metadata::FragmentShader(ref v) => fs_main = &v,
                        &Metadata::DepthTest(v) => render_state.depth_test = v,
                        &Metadata::DepthWrite(v) => render_state.depth_write = v,
                        &Metadata::Blend((eq, src, dst)) => {
                            render_state.color_blend = Some((eq, src, dst))
                        }
                    }
                }
                &Statement::PriorVariable(ref variable) => {
                    match variable.qualifier {
                        Qualifier::Attribute => {
                            let num = match variable.tt {
                                Type::Vec2 => 2,
                                Type::Vec3 => 3,
                                Type::Vec4 => 4,
                                _ => bail!(ErrorKind::NotSupportVertexAttribute),
                            };

                            if let Some(attribute) =
                                graphics::VertexAttribute::from_str(&variable.ident) {
                                layout.with(attribute, num);
                            } else {
                                bail!(ErrorKind::NotSupportVertexAttribute);
                            }
                        }
                        Qualifier::Uniform => {
                            if let Some(tt) = variable.tt.into() {
                                uniforms.insert(variable.ident.clone(), tt);
                            } else {
                                bail!(ErrorKind::NotSupportUniform);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
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

            vs.push(stmt.clone());
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

            fs.push(stmt.clone());
        }

        Ok(Shader {
               vs: vs,
               vs_main: vs_main.to_owned(),
               fs: fs,
               fs_main: fs_main.to_owned(),
               render_state: render_state,
               layout: layout.finish(),
               uniforms: uniforms,
           })
    }

    pub fn statements(&self, phase: ShaderPhase) -> &Vec<Statement> {
        match phase {
            ShaderPhase::Vertex => &self.vs,
            ShaderPhase::Fragment => &self.fs,
        }
    }

    pub fn main(&self, phase: ShaderPhase) -> &str {
        match phase {
            ShaderPhase::Vertex => &self.vs_main,
            ShaderPhase::Fragment => &self.fs_main,
        }
    }

    pub fn render_state(&self) -> &graphics::RenderState {
        &self.render_state
    }

    pub fn layout(&self) -> &graphics::AttributeLayout {
        &self.layout
    }

    pub fn uniforms(&self) -> &HashMap<String, graphics::UniformVariableType> {
        &self.uniforms
    }
}

impl Into<Option<graphics::UniformVariableType>> for Type {
    fn into(self) -> Option<graphics::UniformVariableType> {
        use self::graphics::UniformVariableType as uvt;
        match self {
            Type::Sampler2D => Some(uvt::Texture),
            Type::Int => Some(uvt::I32),
            Type::Float => Some(uvt::F32),
            Type::Vec2 => Some(uvt::Vector2f),
            Type::Vec3 => Some(uvt::Vector3f),
            Type::Vec4 => Some(uvt::Vector4f),
            Type::Mat2 => Some(uvt::Matrix2f),
            Type::Mat3 => Some(uvt::Matrix3f),
            Type::Mat4 => Some(uvt::Matrix4f),
            _ => None,
        }
    }
}