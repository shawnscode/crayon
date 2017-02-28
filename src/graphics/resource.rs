use super::{TextureHandle, RenderBufferHandle};

pub const MAX_ATTRIBUTES: usize = 12;
pub const MAX_TEXTURE_SLOTS: usize = 16;

/// Specifies the target to which the buffer object is bound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    /// Vertex attributes.
    Vertex,
    /// Vertex array indices.
    Index,
}

/// Hint abouts how this memory will be used.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceHint {
    /// Full speed GPU access. Optimal for render targets and resourced memory.
    Static,
    /// CPU to GPU data flow with update commands.
    /// Used for dynamic buffer data, typically constant buffers.
    Dynamic,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IndexFormat {
    UByte,
    UShort,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexFormat {
    Byte,
    UByte,
    Short,
    UShort,
    Fixed,
    Float,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color0 = 4,
    Color1 = 5,
    Indices = 6,
    Weight = 7,
    Texcoord0 = 8,
    Texcoord1 = 9,
    Texcoord2 = 10,
    Texcoord3 = 11,
}

impl Into<&'static str> for VertexAttribute {
    fn into(self) -> &'static str {
        match self {
            VertexAttribute::Position => "Position",
            VertexAttribute::Normal => "Normal",
            VertexAttribute::Tangent => "Tangent",
            VertexAttribute::Bitangent => "Bitangent",
            VertexAttribute::Color0 => "Color0",
            VertexAttribute::Color1 => "Color1",
            VertexAttribute::Indices => "Indices",
            VertexAttribute::Weight => "Weight",
            VertexAttribute::Texcoord0 => "Texcoord0",
            VertexAttribute::Texcoord1 => "Texcoord1",
            VertexAttribute::Texcoord2 => "Texcoord2",
            VertexAttribute::Texcoord3 => "Texcoord3",
        }
    }
}

// VertexAttribute defines an generic vertex element data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VertexAttributeDesc {
    /// The name of this description.
    pub name: VertexAttribute,
    /// The data type of each component of this element.
    pub format: VertexFormat,
    /// The number of components per generic vertex element.
    pub size: u8,
    /// Whether fixed-point data values should be normalized.
    pub normalized: bool,
}

impl Default for VertexAttributeDesc {
    fn default() -> Self {
        VertexAttributeDesc {
            name: VertexAttribute::Position,
            format: VertexFormat::Byte,
            size: 0,
            normalized: false,
        }
    }
}

// VertexLayout defines an layout of vertex structure.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct VertexLayout {
    stride: u8,
    len: u8,
    offset: [u8; MAX_ATTRIBUTES],
    elements: [VertexAttributeDesc; MAX_ATTRIBUTES],
}

impl VertexLayout {
    /// Creates a new an empty `VertexLayoutBuilder`.
    #[inline]
    pub fn build() -> VertexLayoutBuilder {
        VertexLayoutBuilder::new()
    }

    /// Stride of single vertex structure.
    #[inline]
    pub fn stride(&self) -> u8 {
        self.stride
    }

    /// Returns the number of elements in the layout.
    #[inline]
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Relative element offset from the layout.
    pub fn offset(&self, name: VertexAttribute) -> Option<u8> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.offset[i]),
                _ => (),
            }
        }

        None
    }

    /// Returns named `VertexAttribute` from the layout.
    pub fn element(&self, name: VertexAttribute) -> Option<VertexAttributeDesc> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.elements[i]),
                _ => (),
            }
        }

        None
    }
}

#[derive(Default)]
pub struct VertexLayoutBuilder(VertexLayout);

impl VertexLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(&mut self,
                attribute: VertexAttribute,
                format: VertexFormat,
                size: u8,
                normalized: bool)
                -> &mut Self {
        assert!(size > 0 && size <= 4);

        let desc = VertexAttributeDesc {
            name: attribute,
            format: format,
            size: size,
            normalized: normalized,
        };

        for i in 0..self.0.len {
            let i = i as usize;
            if self.0.elements[i].name == attribute {
                self.0.elements[i] = desc;
                return self;
            }
        }

        assert!((self.0.len as usize) < MAX_ATTRIBUTES);
        self.0.elements[self.0.len as usize] = desc;
        self.0.len += 1;

        self
    }

    #[inline]
    pub fn finish(&mut self) -> VertexLayout {
        self.0.stride = 0;
        for i in 0..self.0.len {
            let i = i as usize;
            let len = self.0.elements[i].size * size_of_vertex(self.0.elements[i].format);
            self.0.offset[i] = self.0.stride;
            self.0.stride += len;
        }
        self.0
    }
}

fn size_of_vertex(format: VertexFormat) -> u8 {
    match format {
        VertexFormat::Byte | VertexFormat::UByte => 1,
        VertexFormat::Short | VertexFormat::UShort | VertexFormat::Fixed => 2,
        VertexFormat::Float => 4,
    }
}

/// Specify how the texture is used whenever the pixel being sampled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFilter {
    /// Returns the value of the texture element that is nearest (in Manhattan distance)
    /// to the center of the pixel being textured.
    Nearest,
    /// Returns the weighted average of the four texture elements that are closest to the
    /// center of the pixel being textured.
    Linear,
}

/// Sets the wrap parameter for texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureAddress {
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

/// List of all the possible formats of input data when uploading to texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    U5U6U5,
    U4U4U4U4,
    U5U5U5U1,
    U10U10U10U2,
    F16,
    F16F16,
    F16F16F16,
    F16F16F16F16,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
}

impl TextureFormat {
    /// Returns the number of components of this client format.
    pub fn components(&self) -> u8 {
        match *self {
            TextureFormat::U8 => 1,
            TextureFormat::U8U8 => 2,
            TextureFormat::U8U8U8 => 3,
            TextureFormat::U8U8U8U8 => 4,
            TextureFormat::U5U6U5 => 3,
            TextureFormat::U4U4U4U4 => 4,
            TextureFormat::U5U5U5U1 => 4,
            TextureFormat::U10U10U10U2 => 4,
            TextureFormat::F16 => 1,
            TextureFormat::F16F16 => 2,
            TextureFormat::F16F16F16 => 3,
            TextureFormat::F16F16F16F16 => 4,
            TextureFormat::F32 => 1,
            TextureFormat::F32F32 => 2,
            TextureFormat::F32F32F32 => 3,
            TextureFormat::F32F32F32F32 => 4,
        }
    }

    /// Returns the size in bytes of a pixel of this type.
    pub fn size(&self) -> u8 {
        match *self {
            TextureFormat::U8 => 1,
            TextureFormat::U8U8 => 2,
            TextureFormat::U8U8U8 => 3,
            TextureFormat::U8U8U8U8 => 4,
            TextureFormat::U5U6U5 => 2,
            TextureFormat::U4U4U4U4 => 2,
            TextureFormat::U5U5U5U1 => 2,
            TextureFormat::U10U10U10U2 => 4,
            TextureFormat::F16 => 2,
            TextureFormat::F16F16 => 4,
            TextureFormat::F16F16F16 => 6,
            TextureFormat::F16F16F16F16 => 8,
            TextureFormat::F32 => 4,
            TextureFormat::F32F32 => 8,
            TextureFormat::F32F32F32 => 12,
            TextureFormat::F32F32F32F32 => 16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameBufferAttachment {
    Texture(TextureHandle),
    RenderBuffer(RenderBufferHandle),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let layout = VertexLayout::build()
            .with(VertexAttribute::Position, VertexFormat::Float, 3, true)
            .with(VertexAttribute::Texcoord0, VertexFormat::Float, 2, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
        assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(VertexAttribute::Normal), None);

        let element = layout.element(VertexAttribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(VertexAttribute::Normal), None);
    }

    #[test]
    fn rewrite() {
        let layout = VertexLayout::build()
            .with(VertexAttribute::Position, VertexFormat::Fixed, 1, false)
            .with(VertexAttribute::Texcoord0, VertexFormat::Float, 2, true)
            .with(VertexAttribute::Position, VertexFormat::Float, 3, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
        assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(VertexAttribute::Normal), None);

        let element = layout.element(VertexAttribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(VertexAttribute::Normal), None);
    }
}