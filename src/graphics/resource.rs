use std::borrow::Borrow;

use utility::hash::*;
use super::MAX_ATTRIBUTES;

/// Specifies the target to which the buffer object is bound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    /// Vertex attributes.
    Vertex,
    /// Vertex array indices.
    Index,
}

/// Hint abouts how this memory will be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceHint {
    /// Full speed GPU access. Optimal for render targets and resourced memory.
    Static,
    /// CPU to GPU data flow with update commands.
    /// Used for dynamic buffer data, typically constant buffers.
    Dynamic,
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

// VertexAttribute defines an generic vertex element data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VertexAttribute {
    /// Hash value of the element name.
    pub name: u64,
    /// The data type of each component of this element.
    pub format: VertexFormat,
    /// The number of components per generic vertex element.
    pub size: u8,
    /// Whether fixed-point data values should be normalized.
    pub normalized: bool,
}

impl Default for VertexAttribute {
    fn default() -> Self {
        VertexAttribute {
            name: 0,
            format: VertexFormat::Byte,
            size: 0,
            normalized: false,
        }
    }
}

// VertexLayout defines an layout of vertex structure.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct VertexLayout {
    stride: u32,
    offset: [u32; MAX_ATTRIBUTES],
    elements: [VertexAttribute; MAX_ATTRIBUTES],
}

impl VertexLayout {
    /// Creates a new an empty `VertexLayoutBuilder`.
    #[inline]
    pub fn build() -> VertexLayoutBuilder {
        VertexLayoutBuilder::new()
    }

    /// Stride of single vertex structure.
    pub fn stride(&self) -> u32 {
        self.stride
    }

    /// Relative element offset from the layout.
    pub fn offset<T: Borrow<str>>(&self, name: T) -> Option<u32> {
        let h = hash(&name.borrow());

        for i in 0..self.elements.len() {
            match self.elements[i].name {
                0 => break,
                v if v == h => return Some(self.offset[i]),
                _ => (),
            }
        }

        None
    }

    /// Returns named `VertexAttribute` from the layout.
    pub fn element<T: Borrow<str>>(&self, name: T) -> Option<VertexAttribute> {
        let h = hash(&name.borrow());
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                0 => break,
                v if v == h => return Some(self.elements[i]),
                _ => (),
            }
        }

        None
    }
}

pub struct VertexLayoutBuilder {
    layout: VertexLayout,
}

impl VertexLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        VertexLayoutBuilder { layout: Default::default() }
    }

    pub fn with<T: Borrow<str>>(&mut self,
                                name: T,
                                format: VertexFormat,
                                size: u8,
                                normalized: bool)
                                -> &mut Self {
        let h = hash(&name.borrow());
        for i in 0..self.layout.elements.len() {
            if self.layout.elements[i].name == h || self.layout.elements[i].name == 0 {
                {
                    let mut element = &mut self.layout.elements[i];
                    element.name = h;
                    element.format = format;
                    element.size = size;
                    element.normalized = normalized;
                }

                return self;
            }
        }

        unreachable!("Out of layout bounds.");
    }

    #[inline]
    pub fn finish(&mut self) -> VertexLayout {
        self.layout.stride = 0;
        for i in 0..self.layout.elements.len() {
            let v = &self.layout.elements[i];
            if v.name == 0 {
                break;
            }

            let len = v.size as u32 * size(v.format);
            self.layout.offset[i] = self.layout.stride;
            self.layout.stride += len;
        }

        self.layout
    }
}

fn size(format: VertexFormat) -> u32 {
    match format {
        VertexFormat::Byte | VertexFormat::UByte => 1,
        VertexFormat::Short | VertexFormat::UShort | VertexFormat::Fixed => 2,
        VertexFormat::Float => 4,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let layout = VertexLayout::build()
            .with("Position", VertexFormat::Float, 3, true)
            .with("Texcoord", VertexFormat::Float, 2, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset("Position"), Some(0));
        assert_eq!(layout.offset("Texcoord"), Some(12));
        assert_eq!(layout.offset("Normal"), None);

        let element = layout.element("Position").unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element("Normal"), None);
    }

    #[test]
    fn rewrite() {
        let layout = VertexLayout::build()
            .with("Position", VertexFormat::Fixed, 1, false)
            .with("Texcoord", VertexFormat::Float, 2, true)
            .with("Position", VertexFormat::Float, 3, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset("Position"), Some(0));
        assert_eq!(layout.offset("Texcoord"), Some(12));
        assert_eq!(layout.offset("Normal"), None);

        let element = layout.element("Position").unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element("Normal"), None);
    }

    #[test]
    #[should_panic]
    fn too_many_elements() {
        let mut builder = VertexLayout::build();
        for i in 0..MAX_ATTRIBUTES + 1 {
            builder.with(format!("Element_{}", i), VertexFormat::Byte, 1, true);
        }
        builder.finish();
    }
}