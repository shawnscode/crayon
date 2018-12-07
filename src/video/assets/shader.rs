//! Pipeline state object that containing immutable render state and vertex-layout.

use std::collections::hash_map::Values;
use std::str::FromStr;

use crate::math::prelude::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use crate::utils::prelude::{FastHashMap, HashValue};
use crate::video::assets::mesh::VertexLayout;
use crate::video::assets::texture::{RenderTextureHandle, TextureHandle};
use crate::video::errors::{Error, Result};
use crate::video::{MAX_UNIFORM_VARIABLES, MAX_VERTEX_ATTRIBUTES};

impl_handle!(ShaderHandle);

/// A `ShaderParams` encapusulate all the informations we need to configurate
/// OpenGL before real drawing, like shaders, render states, etc.
#[derive(Debug, Clone, Default)]
pub struct ShaderParams {
    pub attributes: AttributeLayout,
    pub uniforms: UniformVariableLayout,
    pub state: RenderState,
}

impl ShaderParams {
    pub fn validate(&self, vs: &str, fs: &str) -> Result<()> {
        if self.uniforms.len() > MAX_UNIFORM_VARIABLES {
            return Err(Error::ShaderInvalid(format!(
                "Too many uniform variables (>= {:?}).",
                MAX_UNIFORM_VARIABLES
            )));
        }

        if vs.is_empty() {
            return Err(Error::ShaderInvalid(
                "Vertex shader is required to describe a proper render pipeline.".into(),
            ));
        }

        if fs.is_empty() {
            return Err(Error::ShaderInvalid(
                "Fragment shader is required to describe a proper render pipeline.".into(),
            ));
        }

        Ok(())
    }
}

/// The possible pre-defined and named attributes in the vertex component, describing
/// what the vertex component is used for.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Attribute {
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

impl Into<&'static str> for Attribute {
    fn into(self) -> &'static str {
        match self {
            Attribute::Position => "Position",
            Attribute::Normal => "Normal",
            Attribute::Tangent => "Tangent",
            Attribute::Bitangent => "Bitangent",
            Attribute::Color0 => "Color0",
            Attribute::Color1 => "Color1",
            Attribute::Indices => "Indices",
            Attribute::Weight => "Weight",
            Attribute::Texcoord0 => "Texcoord0",
            Attribute::Texcoord1 => "Texcoord1",
            Attribute::Texcoord2 => "Texcoord2",
            Attribute::Texcoord3 => "Texcoord3",
        }
    }
}

impl FromStr for Attribute {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Position" => Ok(Attribute::Position),
            "Normal" => Ok(Attribute::Normal),
            "Tangent" => Ok(Attribute::Tangent),
            "Bitangent" => Ok(Attribute::Bitangent),
            "Color0" => Ok(Attribute::Color0),
            "Color1" => Ok(Attribute::Color1),
            "Indices" => Ok(Attribute::Indices),
            "Weight" => Ok(Attribute::Weight),
            "Texcoord0" => Ok(Attribute::Texcoord0),
            "Texcoord1" => Ok(Attribute::Texcoord1),
            "Texcoord2" => Ok(Attribute::Texcoord2),
            "Texcoord3" => Ok(Attribute::Texcoord3),
            _ => Err(Error::AttributeUndefined(s.into())),
        }
    }
}

// AttributeLayout defines an layout of attributes into program.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AttributeLayout {
    len: u8,
    elements: [(Attribute, u8, bool); MAX_VERTEX_ATTRIBUTES],
}

impl Default for AttributeLayout {
    fn default() -> Self {
        AttributeLayout {
            len: 0,
            elements: [(Attribute::Position, 0, false); MAX_VERTEX_ATTRIBUTES],
        }
    }
}

impl AttributeLayout {
    pub fn build() -> AttributeLayoutBuilder {
        AttributeLayoutBuilder::new()
    }

    pub fn iter(&self) -> AttributeLayoutIter {
        AttributeLayoutIter {
            pos: 0,
            layout: self,
        }
    }

    pub fn is_match(&self, layout: &VertexLayout) -> bool {
        for (name, size, required) in self.iter() {
            if required {
                if let Some(element) = layout.element(name) {
                    if element.size == size {
                        continue;
                    }
                }
            }

            return false;
        }

        true
    }
}

pub struct AttributeLayoutIter<'a> {
    pos: u8,
    layout: &'a AttributeLayout,
}

impl<'a> Iterator for AttributeLayoutIter<'a> {
    type Item = (Attribute, u8, bool);

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

    #[inline]
    pub fn with(self, attribute: Attribute, size: u8) -> Self {
        self.append(attribute, size, true)
    }

    #[inline]
    pub fn with_optional(self, attribute: Attribute, size: u8) -> Self {
        self.append(attribute, size, false)
    }

    fn append(mut self, attribute: Attribute, size: u8, required: bool) -> Self {
        assert!(size > 0 && size <= 4);

        for i in 0..self.0.len {
            let i = i as usize;
            if self.0.elements[i].0 == attribute {
                self.0.elements[i] = (attribute, size, required);
                return self;
            }
        }

        assert!((self.0.len as usize) < MAX_VERTEX_ATTRIBUTES);
        self.0.elements[self.0.len as usize] = (attribute, size, required);
        self.0.len += 1;
        self
    }

    #[inline]
    pub fn finish(self) -> AttributeLayout {
        self.0
    }
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendValue {
    SourceColor,
    SourceAlpha,
    DestinationColor,
    DestinationAlpha,
}

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
}

/// A struct that encapsulate all the necessary render states.
#[derive(Debug, PartialEq, Clone, Copy)]
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
            depth_write: false,             // no depth write,
            depth_write_offset: None,
            color_blend: None,
            color_write: (true, true, true, true),
        }
    }
}

/// Uniform variable type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UniformVariableType {
    Texture,
    RenderTexture,
    I32,
    F32,
    Vector2f,
    Vector3f,
    Vector4f,
    Matrix2f,
    Matrix3f,
    Matrix4f,
}

/// Uniform variable for video program object. Each matrix based `UniformVariable`
/// is assumed to be supplied in row major order with a optional transpose.
#[derive(Debug, Copy, Clone)]
pub enum UniformVariable {
    Texture(TextureHandle),
    RenderTexture(RenderTextureHandle),
    I32(i32),
    F32(f32),
    Vector2f([f32; 2]),
    Vector3f([f32; 3]),
    Vector4f([f32; 4]),
    Matrix2f([[f32; 2]; 2], bool),
    Matrix3f([[f32; 3]; 3], bool),
    Matrix4f([[f32; 4]; 4], bool),
}

impl UniformVariable {
    pub fn variable_type(&self) -> UniformVariableType {
        match *self {
            UniformVariable::RenderTexture(_) => UniformVariableType::RenderTexture,
            UniformVariable::Texture(_) => UniformVariableType::Texture,
            UniformVariable::I32(_) => UniformVariableType::I32,
            UniformVariable::F32(_) => UniformVariableType::F32,
            UniformVariable::Vector2f(_) => UniformVariableType::Vector2f,
            UniformVariable::Vector3f(_) => UniformVariableType::Vector3f,
            UniformVariable::Vector4f(_) => UniformVariableType::Vector4f,
            UniformVariable::Matrix2f(_, _) => UniformVariableType::Matrix2f,
            UniformVariable::Matrix3f(_, _) => UniformVariableType::Matrix3f,
            UniformVariable::Matrix4f(_, _) => UniformVariableType::Matrix4f,
        }
    }
}

impl Into<UniformVariable> for TextureHandle {
    fn into(self) -> UniformVariable {
        UniformVariable::Texture(self)
    }
}

impl Into<UniformVariable> for RenderTextureHandle {
    fn into(self) -> UniformVariable {
        UniformVariable::RenderTexture(self)
    }
}

impl Into<UniformVariable> for i32 {
    fn into(self) -> UniformVariable {
        UniformVariable::I32(self)
    }
}

impl Into<UniformVariable> for f32 {
    fn into(self) -> UniformVariable {
        UniformVariable::F32(self)
    }
}

impl Into<UniformVariable> for Matrix2<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix2f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for [[f32; 2]; 2] {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix2f(self, false)
    }
}

impl Into<UniformVariable> for Matrix3<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix3f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for [[f32; 3]; 3] {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix3f(self, false)
    }
}

impl Into<UniformVariable> for Matrix4<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix4f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for [[f32; 4]; 4] {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix4f(self, false)
    }
}

impl Into<UniformVariable> for Vector2<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector2f(*self.as_ref())
    }
}

impl Into<UniformVariable> for [f32; 2] {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector2f(self)
    }
}

impl Into<UniformVariable> for Vector3<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector3f(*self.as_ref())
    }
}

impl Into<UniformVariable> for [f32; 3] {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector3f(self)
    }
}

impl Into<UniformVariable> for Vector4<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector4f(*self.as_ref())
    }
}

impl Into<UniformVariable> for [f32; 4] {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector4f(self)
    }
}

// UniformVariableLayout defines an layout of uniforms in program.
#[derive(Debug, Clone, Default)]
pub struct UniformVariableLayout {
    variables: FastHashMap<HashValue<str>, (String, UniformVariableType)>,
}

impl UniformVariableLayout {
    pub fn build() -> UniformVariableLayoutBuilder {
        UniformVariableLayoutBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    pub fn iter(&self) -> Values<HashValue<str>, (String, UniformVariableType)> {
        self.variables.values()
    }

    pub fn variable_type<T>(&self, field: T) -> Option<UniformVariableType>
    where
        T: Into<HashValue<str>>,
    {
        self.variables.get(&field.into()).map(|v| v.1)
    }

    pub fn variable_name<T>(&self, field: T) -> Option<&str>
    where
        T: Into<HashValue<str>>,
    {
        self.variables.get(&field.into()).map(|v| v.0.as_ref())
    }
}

#[derive(Default)]
pub struct UniformVariableLayoutBuilder(UniformVariableLayout);

impl UniformVariableLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with<T>(mut self, field: T, v: UniformVariableType) -> Self
    where
        T: Into<String>,
    {
        let field = field.into();
        let hash = HashValue::from(&field);
        self.0.variables.insert(hash, (field, v));
        self
    }

    #[inline]
    pub fn finish(self) -> UniformVariableLayout {
        self.0
    }
}
