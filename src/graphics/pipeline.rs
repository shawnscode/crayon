
use super::*;
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

struct PipelineBuilder {}

impl PipelineBuilder {
    pub fn new() {}

    pub fn with_attribute() -> &mut Self {}
    pub fn with_uniform<T: Borrow<str>>(&mut self, name: T) -> &mut Self {}
    pub fn with_program(&mut self, handle: Handle) -> &mut Self {}

    pub fn with_viewport(&mut self, position: (u32, u32), size: (u32, u32)) -> &mut Self {}
    pub fn with_face_cull(&mut self, face: CullFace) -> &mut Self {}
    pub fn with_front_face(&mut self, front: FrontFaceOrder) -> &mut Self {}
    pub fn with_depth_test(&mut self, comparsion: Comparison) -> &mut Self {}
    pub fn with_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>) -> &mut Self {}
    pub fn with_color_blend(&mut self,
                            enable: bool,
                            equation: Equation,
                            src: BlendFactor,
                            dst: BlendFactor)
                            -> &mut Self {
    }
    pub fn with_color_write(&mut self,
                            red: bool,
                            green: bool,
                            blue: bool,
                            alpha: bool)
                            -> &mut Self {
    }
}