use gl;
use gl::types::*;
use std::borrow::Borrow;

use super::super::super::assets::prelude::*;
use utils::handle;

pub struct DataVec<T>
where
    T: Sized + Clone,
{
    pub buf: Vec<Option<T>>,
}

impl<T> DataVec<T>
where
    T: Sized + Clone,
{
    pub fn new() -> Self {
        DataVec { buf: Vec::new() }
    }

    pub fn get<H>(&self, handle: H) -> Option<&T>
    where
        H: Borrow<handle::Handle>,
    {
        self.buf
            .get(handle.borrow().index() as usize)
            .and_then(|v| v.as_ref())
    }

    pub fn create<H>(&mut self, handle: H, value: T)
    where
        H: Borrow<handle::Handle>,
    {
        let handle = handle.borrow();
        self.buf.resize(handle.index() as usize + 1, None);
        self.buf[handle.index() as usize] = Some(value);
    }

    pub fn free<H>(&mut self, handle: H) -> Option<T>
    where
        H: Borrow<handle::Handle>,
    {
        let handle = handle.borrow();
        if self.buf.len() <= handle.index() as usize {
            None
        } else {
            let mut value = None;
            ::std::mem::swap(&mut value, &mut self.buf[handle.index() as usize]);
            value
        }
    }
}

impl From<MeshHint> for GLenum {
    fn from(hint: MeshHint) -> Self {
        match hint {
            MeshHint::Immutable => gl::STATIC_DRAW,
            MeshHint::Stream => gl::STREAM_DRAW,
            MeshHint::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

impl From<Comparison> for GLenum {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => gl::NEVER,
            Comparison::Less => gl::LESS,
            Comparison::LessOrEqual => gl::LEQUAL,
            Comparison::Greater => gl::GREATER,
            Comparison::GreaterOrEqual => gl::GEQUAL,
            Comparison::Equal => gl::EQUAL,
            Comparison::NotEqual => gl::NOTEQUAL,
            Comparison::Always => gl::ALWAYS,
        }
    }
}

impl From<Equation> for GLenum {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => gl::FUNC_ADD,
            Equation::Subtract => gl::FUNC_SUBTRACT,
            Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for GLenum {
    fn from(factor: BlendFactor) -> Self {
        match factor {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => gl::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => gl::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => gl::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => gl::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => gl::ONE_MINUS_DST_ALPHA,
        }
    }
}

impl From<VertexFormat> for GLenum {
    fn from(format: VertexFormat) -> Self {
        match format {
            VertexFormat::Byte => gl::BYTE,
            VertexFormat::UByte => gl::UNSIGNED_BYTE,
            VertexFormat::Short => gl::SHORT,
            VertexFormat::UShort => gl::UNSIGNED_SHORT,
            VertexFormat::Float => gl::FLOAT,
        }
    }
}

impl From<MeshPrimitive> for GLenum {
    fn from(primitive: MeshPrimitive) -> Self {
        match primitive {
            MeshPrimitive::Points => gl::POINTS,
            MeshPrimitive::Lines => gl::LINES,
            MeshPrimitive::LineStrip => gl::LINE_STRIP,
            MeshPrimitive::Triangles => gl::TRIANGLES,
            MeshPrimitive::TriangleStrip => gl::TRIANGLE_STRIP,
        }
    }
}

impl From<IndexFormat> for GLenum {
    fn from(format: IndexFormat) -> Self {
        match format {
            IndexFormat::U16 => gl::UNSIGNED_SHORT,
            IndexFormat::U32 => gl::UNSIGNED_INT,
        }
    }
}

impl From<TextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::U8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8U8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8U8U8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            TextureFormat::U5U6U5 => (gl::RGB565, gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
            TextureFormat::U4U4U4U4 => (gl::RGBA4, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            TextureFormat::U5U5U5U1 => (gl::RGB5_A1, gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
            TextureFormat::U10U10U10U2 => (gl::RGB10_A2, gl::RGBA, gl::UNSIGNED_INT_2_10_10_10_REV),
            TextureFormat::F16 => (gl::R16F, gl::RED, gl::HALF_FLOAT),
            TextureFormat::F16F16 => (gl::RG16F, gl::RG, gl::HALF_FLOAT),
            TextureFormat::F16F16F16 => (gl::RGB16F, gl::RGB, gl::HALF_FLOAT),
            TextureFormat::F16F16F16F16 => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
            TextureFormat::F32 => (gl::R32F, gl::RED, gl::FLOAT),
            TextureFormat::F32F32 => (gl::RG32F, gl::RG, gl::FLOAT),
            TextureFormat::F32F32F32 => (gl::RGB32F, gl::RGB, gl::FLOAT),
            TextureFormat::F32F32F32F32 => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
        }
    }
}

impl From<TextureWrap> for GLenum {
    fn from(wrap: TextureWrap) -> Self {
        match wrap {
            TextureWrap::Repeat => gl::REPEAT,
            TextureWrap::Mirror => gl::MIRRORED_REPEAT,
            TextureWrap::Clamp => gl::CLAMP_TO_EDGE,
            TextureWrap::MirrorClamp => gl::MIRROR_CLAMP_TO_EDGE,
        }
    }
}

impl From<RenderTextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: RenderTextureFormat) -> Self {
        match format {
            RenderTextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            RenderTextureFormat::RGBA4 => (gl::RGBA4, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            RenderTextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            RenderTextureFormat::Depth16 => (gl::DEPTH_COMPONENT16, gl::DEPTH_COMPONENT, gl::FLOAT),
            RenderTextureFormat::Depth24 => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::FLOAT),
            RenderTextureFormat::Depth32 => (gl::DEPTH_COMPONENT32, gl::DEPTH_COMPONENT, gl::FLOAT),
            RenderTextureFormat::Depth24Stencil8 => {
                (gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_BYTE)
            }
        }
    }
}
