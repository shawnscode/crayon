//! Pipeline state object that containing immutable render state and vertex-layout.

use super::mesh::{VertexAttribute, MAX_ATTRIBUTES};

/// A `PipelineStateObject` encapusulate all the informations we need to configurate
/// OpenGL before real drawing, like shaders, render states, etc.
#[derive(Debug, Default, Copy, Clone)]
pub struct PipelineStateSetup {
    pub layout: AttributeLayout,
    pub state: RenderState,
}

impl_handle!(PipelineStateHandle);

// AttributeLayout defines an layout of attributes into program.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct AttributeLayout {
    len: u8,
    elements: [(VertexAttribute, u8); MAX_ATTRIBUTES],
}

impl Default for AttributeLayout {
    fn default() -> Self {
        AttributeLayout {
            len: 0,
            elements: [(VertexAttribute::Position, 0); MAX_ATTRIBUTES],
        }
    }
}

impl AttributeLayout {
    pub fn iter(&self) -> AttributeLayoutIter {
        AttributeLayoutIter {
            pos: 0,
            layout: &self,
        }
    }
}

pub struct AttributeLayoutIter<'a> {
    pos: u8,
    layout: &'a AttributeLayout,
}

impl<'a> Iterator for AttributeLayoutIter<'a> {
    type Item = (VertexAttribute, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.layout.len {
            None
        } else {
            self.pos += 1;
            Some(self.layout.elements[self.pos as usize - 1])
        }
    }
}

#[derive(Default)]
pub struct AttributeLayoutBuilder(AttributeLayout);

impl AttributeLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(&mut self, attribute: VertexAttribute, size: u8) -> &mut Self {
        assert!(size > 0 && size <= 4);

        for i in 0..self.0.len {
            let i = i as usize;
            if self.0.elements[i].0 == attribute {
                self.0.elements[i] = (attribute, size);
                return self;
            }
        }

        assert!((self.0.len as usize) < MAX_ATTRIBUTES);
        self.0.elements[self.0.len as usize] = (attribute, size);
        self.0.len += 1;
        self
    }

    #[inline]
    pub fn finish(&mut self) -> AttributeLayout {
        self.0
    }
}

/// Defines how the input vertex data is used to assemble primitives.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Primitive {
    /// Separate points.
    Points,
    /// Separate lines.
    Lines,
    /// Line strips.
    LineStrip,
    /// Separate triangles.
    Triangles,
    /// Triangle strips.
    TriangleStrip,
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Comparison {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Always,
}

/// Specifies how incoming RGBA values (source) and the RGBA in framebuffer (destination)
/// are combined.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Equation {
    /// Adds source and destination. Source and destination are multiplied
    /// by blending parameters before addition.
    Add,
    /// Subtracts destination from source. Source and destination are
    /// multiplied by blending parameters before subtraction.
    Subtract,
    /// Subtracts source from destination. Source and destination are
    /// multiplied by blending parameters before subtraction.
    ReverseSubtract,
}

/// Blend values.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BlendValue {
    SourceColor,
    SourceAlpha,
    DestinationColor,
    DestinationAlpha,
}

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
}

/// A struct that encapsulate all the necessary render states.
#[derive(Debug, PartialEq, Clone, Copy, Builder, Serialize, Deserialize)]
#[builder(field(private))]
pub struct RenderState {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    pub color_blend: Option<(Equation, BlendFactor, BlendFactor)>,
    pub color_write: (bool, bool, bool, bool),
}

impl Default for RenderState {
    fn default() -> Self {
        RenderState {
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false, // no depth write,
            depth_write_offset: None,
            color_blend: None,
            color_write: (false, false, false, false),
        }
    }
}