use super::buffer::*;

pub struct ShaderDescriptor<'a> {
    pub vs: &'a str,
    pub fs: &'a str,
    pub attributes: Vec<(String, VertexFormat, u8)>,
}