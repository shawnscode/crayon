use math;

/// Specifies what kind of primitives to render.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Primitive {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
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
            cull_face: CullFace::Back,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false, // no depth write,
            depth_write_offset: None,
            color_blend: None,
            color_write: (false, false, false, false),
        }
    }
}

/// Uniform variable for graphics program object. Each matrix based `UniformVariable`
/// is assumed to be supplied in row major order with a optional transpose.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UniformVariable {
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
        match self {
            &UniformVariable::I32(_) => UniformVariableType::I32,
            &UniformVariable::F32(_) => UniformVariableType::F32,
            &UniformVariable::Vector2f(_) => UniformVariableType::Vector2f,
            &UniformVariable::Vector3f(_) => UniformVariableType::Vector3f,
            &UniformVariable::Vector4f(_) => UniformVariableType::Vector4f,
            &UniformVariable::Matrix2f(_, _) => UniformVariableType::Matrix2f,
            &UniformVariable::Matrix3f(_, _) => UniformVariableType::Matrix3f,
            &UniformVariable::Matrix4f(_, _) => UniformVariableType::Matrix4f,
        }
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

impl Into<UniformVariable> for math::Matrix2<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix2f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for math::Matrix3<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix3f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for math::Matrix4<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Matrix4f(*self.as_ref(), false)
    }
}

impl Into<UniformVariable> for math::Vector2<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector2f(*self.as_ref())
    }
}

impl Into<UniformVariable> for math::Vector3<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector3f(*self.as_ref())
    }
}

impl Into<UniformVariable> for math::Vector4<f32> {
    fn into(self) -> UniformVariable {
        UniformVariable::Vector4f(*self.as_ref())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniformVariableType {
    Texture,
    I32,
    F32,
    Vector2f,
    Vector3f,
    Vector4f,
    Matrix2f,
    Matrix3f,
    Matrix4f,
}