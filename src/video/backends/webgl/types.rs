use web_sys::WebGl2RenderingContext as WebGL;

use super::super::super::assets::prelude::*;

impl From<MeshHint> for u32 {
    fn from(hint: MeshHint) -> Self {
        match hint {
            MeshHint::Immutable => WebGL::STATIC_DRAW,
            MeshHint::Stream => WebGL::STREAM_DRAW,
            MeshHint::Dynamic => WebGL::DYNAMIC_DRAW,
        }
    }
}

impl From<VertexFormat> for u32 {
    fn from(format: VertexFormat) -> Self {
        match format {
            VertexFormat::Byte => WebGL::BYTE,
            VertexFormat::UByte => WebGL::UNSIGNED_BYTE,
            VertexFormat::Short => WebGL::SHORT,
            VertexFormat::UShort => WebGL::UNSIGNED_SHORT,
            VertexFormat::Float => WebGL::FLOAT,
        }
    }
}

impl From<MeshPrimitive> for u32 {
    fn from(primitive: MeshPrimitive) -> Self {
        match primitive {
            MeshPrimitive::Points => WebGL::POINTS,
            MeshPrimitive::Lines => WebGL::LINES,
            MeshPrimitive::LineStrip => WebGL::LINE_STRIP,
            MeshPrimitive::Triangles => WebGL::TRIANGLES,
            MeshPrimitive::TriangleStrip => WebGL::TRIANGLE_STRIP,
        }
    }
}

impl From<IndexFormat> for u32 {
    fn from(format: IndexFormat) -> Self {
        match format {
            IndexFormat::U16 => WebGL::UNSIGNED_SHORT,
            IndexFormat::U32 => WebGL::UNSIGNED_INT,
        }
    }
}

impl From<Comparison> for u32 {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => WebGL::NEVER,
            Comparison::Less => WebGL::LESS,
            Comparison::LessOrEqual => WebGL::LEQUAL,
            Comparison::Greater => WebGL::GREATER,
            Comparison::GreaterOrEqual => WebGL::GEQUAL,
            Comparison::Equal => WebGL::EQUAL,
            Comparison::NotEqual => WebGL::NOTEQUAL,
            Comparison::Always => WebGL::ALWAYS,
        }
    }
}

impl From<Equation> for u32 {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => WebGL::FUNC_ADD,
            Equation::Subtract => WebGL::FUNC_SUBTRACT,
            Equation::ReverseSubtract => WebGL::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for u32 {
    fn from(factor: BlendFactor) -> Self {
        match factor {
            BlendFactor::Zero => WebGL::ZERO,
            BlendFactor::One => WebGL::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => WebGL::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => WebGL::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => WebGL::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => WebGL::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => WebGL::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => WebGL::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => WebGL::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => WebGL::ONE_MINUS_DST_ALPHA,
        }
    }
}

impl From<TextureWrap> for u32 {
    fn from(wrap: TextureWrap) -> Self {
        match wrap {
            TextureWrap::Repeat => WebGL::REPEAT,
            TextureWrap::Mirror => WebGL::MIRRORED_REPEAT,
            TextureWrap::Clamp => WebGL::CLAMP_TO_EDGE,
            // WebGL does NOT support MIRROR_CLAMP_TO_EDGE
            TextureWrap::MirrorClamp => WebGL::CLAMP_TO_EDGE,
        }
    }
}

impl From<RenderTextureFormat> for (u32, u32, u32) {
    fn from(format: RenderTextureFormat) -> Self {
        // Notes that WebGL does NOT support sized texture format.
        match format {
            RenderTextureFormat::RGB8 => (WebGL::RGB, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            RenderTextureFormat::RGBA4 => (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_SHORT_4_4_4_4),
            RenderTextureFormat::RGBA8 => (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_BYTE),
            RenderTextureFormat::Depth16 => {
                (WebGL::DEPTH_COMPONENT, WebGL::DEPTH_COMPONENT, WebGL::FLOAT)
            }
            RenderTextureFormat::Depth24 => {
                (WebGL::DEPTH_COMPONENT, WebGL::DEPTH_COMPONENT, WebGL::FLOAT)
            }
            RenderTextureFormat::Depth32 => {
                (WebGL::DEPTH_COMPONENT, WebGL::DEPTH_COMPONENT, WebGL::FLOAT)
            }
            RenderTextureFormat::Depth24Stencil8 => (
                WebGL::DEPTH_STENCIL,
                WebGL::DEPTH_STENCIL,
                WebGL::UNSIGNED_BYTE,
            ),
        }
    }
}
