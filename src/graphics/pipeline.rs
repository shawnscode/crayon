
use super::{MAX_VERTEX_ATTRIBUTES, MAX_UNIFORMS};
use super::vertex::VertexAttribute;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UniformFormat {
}

pub struct Uniform {
    pub format: UniformFormat,
}

// struct Pipeline {
//     pub attributes: [VertexAttribute; MAX_VERTEX_ATTRIBUTES],
//     pub uniforms: [Uniform; MAX_UNIFORMS],
// }