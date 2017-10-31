use math;

/// Uniform variable type.
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