//! Immutable or dynamic 2D texture. A texture is a container of one or more images. It
//! can be the source of a texture access from a Shader.
use crate::math::prelude::Vector2;
use crate::video::errors::{Error, Result};

impl_handle!(TextureHandle);

/// The parameters of a texture object.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct TextureParams {
    /// Hint abouts the intended update strategy of the data.
    pub hint: TextureHint,
    /// Sets the wrap parameter for texture.
    pub wrap: TextureWrap,
    /// Specify how the texture is used whenever the pixel being sampled.
    pub filter: TextureFilter,
    /// Sets the format of data.
    pub format: TextureFormat,
    /// Sets the dimensions of texture.
    pub dimensions: Vector2<u32>,
}

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            filter: TextureFilter::Linear,
            hint: TextureHint::Immutable,
            dimensions: Vector2::new(0, 0),
        }
    }
}

impl TextureParams {
    pub fn validate(&self, data: Option<&TextureData>) -> Result<()> {
        if let Some(buf) = data {
            let len = self.format.size(self.dimensions);
            if !buf.bytes.is_empty() && buf.bytes[0].len() > len as usize {
                return Err(Error::OutOfBounds);
            }
        }

        Ok(())
    }
}

/// Continuous texture data of different mipmap levels.
///
/// Notes that mipmaps are stored in order from largest size to smallest size
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureData {
    pub bytes: Vec<Box<[u8]>>,
}

/// A `RenderTexture` object is basicly texture object with special format. It can
/// be used as a render target. If the `sampler` field is true, it can also be ther
/// source of a texture access from a __shader__.
///
#[derive(Debug, Copy, Clone)]
pub struct RenderTextureParams {
    pub format: RenderTextureFormat,
    pub wrap: TextureWrap,
    pub filter: TextureFilter,
    pub dimensions: Vector2<u32>,
    pub sampler: bool,
}

impl Default for RenderTextureParams {
    fn default() -> Self {
        RenderTextureParams {
            format: RenderTextureFormat::RGB8,
            wrap: TextureWrap::Clamp,
            filter: TextureFilter::Linear,
            dimensions: Vector2::new(0, 0),
            sampler: true,
        }
    }
}

impl_handle!(RenderTextureHandle);

/// Hint abouts the intended update strategy of the data.
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureHint {
    /// The resource is initialized with data and cannot be changed later, this
    /// is the most common and most efficient usage. Optimal for render targets
    /// and resourced memory.
    Immutable,
    /// The resource is initialized without data, but will be be updated by the
    /// CPU in each frame.
    Stream,
    /// The resource is initialized without data and will be written by the CPU
    /// before use, updates will be infrequent.
    Dynamic,
}

/// Specify how the texture is used whenever the pixel being sampled.
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFilter {
    /// Returns the value of the texture element that is nearest (in Manhattan distance)
    /// to the center of the pixel being textured.
    Nearest,
    /// Returns the weighted average of the four texture elements that are closest to the
    /// center of the pixel being textured.
    Linear,
}

/// Sets the wrap parameter for texture.
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureWrap {
    /// Samples at coord x + 1 map to coord x.
    Repeat,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror,
    /// Samples at coord x + 1 map to coord 1.
    Clamp,
    /// Same as Mirror, but only for one repetition.
    MirrorClamp,
}

/// List of all the possible formats of renderable texture which could be use as
/// attachment of framebuffer.
///
/// Each element of `Depth` is a single depth value. The `Graphics` converts it to
/// floating point, multiplies by the signed scale factor, adds the signed bias, and
/// clamps to the range [0,1].
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RenderTextureFormat {
    RGB8,
    RGBA4,
    RGBA8,
    Depth16,
    Depth24,
    Depth32,
    Depth24Stencil8,
}

impl RenderTextureFormat {
    pub fn is_color(self) -> bool {
        self == RenderTextureFormat::RGB8
            || self == RenderTextureFormat::RGBA4
            || self == RenderTextureFormat::RGBA8
    }

    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, dimensions: Vector2<u32>) -> u32 {
        let square = dimensions.x * dimensions.y;
        match self {
            RenderTextureFormat::RGBA4 | RenderTextureFormat::Depth16 => 2 * square,
            RenderTextureFormat::RGB8 | RenderTextureFormat::Depth24 => 3 * square,
            RenderTextureFormat::RGBA8
            | RenderTextureFormat::Depth32
            | RenderTextureFormat::Depth24Stencil8 => 4 * square,
        }
    }
}

/// List of all the possible formats of input data when uploading to texture.
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    RGB565,
    RGBA4,
    RGBA5551,
    RGBA1010102,

    R16F,
    RG16F,
    RGB16F,
    RGBA16F,

    R32F,
    RG32F,
    RGB32F,
    RGBA32F,

    PvrtcRGB4BPP,
    PvrtcRGB2BPP,
    PvrtcRGBA4BPP,
    PvrtcRGBA2BPP,

    Etc2RGB4BPP,
    Etc2RGBA8BPP,

    S3tcDxt1RGB4BPP,
    S3tcDxt5RGBA8BPP,
}

impl TextureFormat {
    /// Returns the number of components of this client format.
    pub fn components(self) -> u8 {
        match self {
            TextureFormat::R32F | TextureFormat::R16F | TextureFormat::R8 => 1,
            TextureFormat::RG8 | TextureFormat::RG16F | TextureFormat::RG32F => 2,
            TextureFormat::RGB565
            | TextureFormat::RGB8
            | TextureFormat::RGB16F
            | TextureFormat::RGB32F
            | TextureFormat::PvrtcRGB4BPP
            | TextureFormat::PvrtcRGB2BPP
            | TextureFormat::Etc2RGB4BPP
            | TextureFormat::S3tcDxt1RGB4BPP => 3,
            TextureFormat::RGBA8
            | TextureFormat::RGBA4
            | TextureFormat::RGBA5551
            | TextureFormat::RGBA1010102
            | TextureFormat::RGBA16F
            | TextureFormat::RGBA32F
            | TextureFormat::PvrtcRGBA4BPP
            | TextureFormat::PvrtcRGBA2BPP
            | TextureFormat::Etc2RGBA8BPP
            | TextureFormat::S3tcDxt5RGBA8BPP => 4,
        }
    }

    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, dimensions: Vector2<u32>) -> u32 {
        let square = dimensions.x * dimensions.y;
        match self {
            TextureFormat::PvrtcRGB2BPP | TextureFormat::PvrtcRGBA2BPP => square / 4,
            TextureFormat::PvrtcRGB4BPP | TextureFormat::PvrtcRGBA4BPP => square / 2,
            TextureFormat::Etc2RGB4BPP | TextureFormat::S3tcDxt1RGB4BPP => square / 2,
            TextureFormat::S3tcDxt5RGBA8BPP => square,
            TextureFormat::Etc2RGBA8BPP => square,
            TextureFormat::R8 => square,
            TextureFormat::RG8
            | TextureFormat::RGB565
            | TextureFormat::RGBA4
            | TextureFormat::RGBA5551
            | TextureFormat::R16F => 2 * square,
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8
            | TextureFormat::RGBA1010102
            | TextureFormat::RG16F
            | TextureFormat::R32F => 4 * square,
            TextureFormat::RGB16F => 6 * square,
            TextureFormat::RGBA16F | TextureFormat::RG32F => 8 * square,
            TextureFormat::RGB32F => 12 * square,
            TextureFormat::RGBA32F => 16 * square,
        }
    }

    pub fn compressed(self) -> bool {
        match self {
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
