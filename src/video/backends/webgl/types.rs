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

impl From<TextureFormat> for (u32, u32, u32) {
    fn from(format: TextureFormat) -> Self {
        // FIXME
        // WebGL::COMPRESSED_RGB_S3TC_DXT1_EXT = 0x83F0
        // WebGL::COMPRESSED_RGBA_S3TC_DXT5_EXT = 0x83F3
        // WebGL::COMPRESSED_RGB_PVRTC_2BPPV1_IMG = 0x8C01
        // WebGL::COMPRESSED_RGB_PVRTC_4BPPV1_IMG = 0x8C00
        // WebGL::COMPRESSED_RGBA_PVRTC_2BPPV1_IMG = 0x8C03
        // WebGL::COMPRESSED_RGBA_PVRTC_4BPPV1_IMG = 0x8C02
        // WebGL::COMPRESSED_RGB8_ETC2 = 0x9274
        // WebGL::COMPRESSED_RGBA8_ETC2_EAC = 0x9278
        match format {
            TextureFormat::R8 => (WebGL::RED, WebGL::RED, WebGL::UNSIGNED_BYTE),
            TextureFormat::RG8 => (WebGL::RG, WebGL::RG, WebGL::UNSIGNED_BYTE),
            TextureFormat::RGB8 => (WebGL::RGB, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_BYTE),
            TextureFormat::RGB565 => (WebGL::RGB, WebGL::RGB, WebGL::UNSIGNED_SHORT_5_6_5),
            TextureFormat::RGBA4 => (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_SHORT_4_4_4_4),
            TextureFormat::RGBA5551 => (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_SHORT_5_5_5_1),
            TextureFormat::RGBA1010102 => {
                (WebGL::RGBA, WebGL::RGBA, WebGL::UNSIGNED_INT_2_10_10_10_REV)
            }
            TextureFormat::R16F => (WebGL::RED, WebGL::RED, WebGL::HALF_FLOAT),
            TextureFormat::RG16F => (WebGL::RG, WebGL::RG, WebGL::HALF_FLOAT),
            TextureFormat::RGB16F => (WebGL::RGB, WebGL::RGB, WebGL::HALF_FLOAT),
            TextureFormat::RGBA16F => (WebGL::RGBA, WebGL::RGBA, WebGL::HALF_FLOAT),
            TextureFormat::R32F => (WebGL::RED, WebGL::RED, WebGL::FLOAT),
            TextureFormat::RG32F => (WebGL::RG, WebGL::RG, WebGL::FLOAT),
            TextureFormat::RGB32F => (WebGL::RGB, WebGL::RGB, WebGL::FLOAT),
            TextureFormat::RGBA32F => (WebGL::RGBA, WebGL::RGBA, WebGL::FLOAT),
            TextureFormat::Etc2RGB4BPP => (0x9274, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::Etc2RGBA8BPP => (0x9278, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::S3tcDxt1RGB4BPP => (0x83F0, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::S3tcDxt5RGBA8BPP => (0x83F3, WebGL::RGBA, WebGL::UNSIGNED_BYTE),
            TextureFormat::PvrtcRGB2BPP => (0x8C01, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::PvrtcRGB4BPP => (0x8C00, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::PvrtcRGBA2BPP => (0x8C03, WebGL::RGB, WebGL::UNSIGNED_BYTE),
            TextureFormat::PvrtcRGBA4BPP => (0x8C02, WebGL::RGB, WebGL::UNSIGNED_BYTE),
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
