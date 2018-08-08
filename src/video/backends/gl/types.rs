use gl;
use gl::types::*;
use std::borrow::Borrow;

use super::super::super::assets::prelude::*;
use super::capabilities::{Capabilities, TextureCompression};
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

    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
    where
        H: Borrow<handle::Handle>,
    {
        self.buf
            .get_mut(handle.borrow().index() as usize)
            .and_then(|v| v.as_mut())
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
        // Notes that OpenGL ES 2.0 does NOT supports sized internal format.
        match format {
            TextureFormat::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            TextureFormat::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
            TextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            TextureFormat::RGB565 => (gl::RGB565, gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
            TextureFormat::RGBA4 => (gl::RGBA4, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            TextureFormat::RGBA5551 => (gl::RGB5_A1, gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
            TextureFormat::RGBA1010102 => (gl::RGB10_A2, gl::RGBA, gl::UNSIGNED_INT_2_10_10_10_REV),
            TextureFormat::R16F => (gl::R16F, gl::RED, gl::HALF_FLOAT),
            TextureFormat::RG16F => (gl::RG16F, gl::RG, gl::HALF_FLOAT),
            TextureFormat::RGB16F => (gl::RGB16F, gl::RGB, gl::HALF_FLOAT),
            TextureFormat::RGBA16F => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
            TextureFormat::R32F => (gl::R32F, gl::RED, gl::FLOAT),
            TextureFormat::RG32F => (gl::RG32F, gl::RG, gl::FLOAT),
            TextureFormat::RGB32F => (gl::RGB32F, gl::RGB, gl::FLOAT),
            TextureFormat::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
            TextureFormat::Etc2RGB4BPP => (gl::COMPRESSED_RGB8_ETC2, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormat::Etc2RGBA8BPP => {
                (gl::COMPRESSED_RGBA8_ETC2_EAC, gl::RGB, gl::UNSIGNED_BYTE)
            }
            TextureFormat::S3tcDxt1RGB4BPP => (
                // FIXME
                // gl::COMPRESSED_RGB_S3TC_DXT1_EXT,
                0x83F0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            TextureFormat::S3tcDxt5RGBA8BPP => (
                // gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
                0x83F3,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
            ),
            TextureFormat::PvrtcRGB2BPP => (
                // gl::COMPRESSED_RGB_PVRTC_2BPPV1_IMG,
                0x8C01,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            TextureFormat::PvrtcRGB4BPP => (
                // gl::COMPRESSED_RGB_PVRTC_4BPPV1_IMG,
                0x8C00,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            TextureFormat::PvrtcRGBA2BPP => (
                // gl::COMPRESSED_RGBA_PVRTC_2BPPV1_IMG,
                0x8C03,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            TextureFormat::PvrtcRGBA4BPP => (
                // gl::COMPRESSED_RGBA_PVRTC_4BPPV1_IMG,
                0x8C02,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
        }
    }
}

impl TextureFormat {
    pub fn is_support(&self, capabilities: &Capabilities) -> bool {
        match *self {
            TextureFormat::Etc2RGB4BPP | TextureFormat::Etc2RGBA8BPP => {
                capabilities.has_compression(TextureCompression::ETC2)
            }
            TextureFormat::PvrtcRGB2BPP
            | TextureFormat::PvrtcRGB4BPP
            | TextureFormat::PvrtcRGBA2BPP
            | TextureFormat::PvrtcRGBA4BPP => {
                capabilities.has_compression(TextureCompression::PVRTC)
            }
            TextureFormat::S3tcDxt1RGB4BPP | TextureFormat::S3tcDxt5RGBA8BPP => {
                capabilities.has_compression(TextureCompression::S3TC)
            }
            _ => true,
        }
    }

    pub fn is_compression(&self) -> bool {
        match *self {
            TextureFormat::Etc2RGB4BPP
            | TextureFormat::Etc2RGBA8BPP
            | TextureFormat::PvrtcRGB2BPP
            | TextureFormat::PvrtcRGB4BPP
            | TextureFormat::PvrtcRGBA2BPP
            | TextureFormat::PvrtcRGBA4BPP
            | TextureFormat::S3tcDxt1RGB4BPP
            | TextureFormat::S3tcDxt5RGBA8BPP => true,
            _ => false,
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
