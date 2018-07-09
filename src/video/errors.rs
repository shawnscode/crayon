#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Backend(String),
    #[fail(display = "Failed to compile shader, errors:\n{}\nsource:\n{}\n", errors, source)]
    ShaderSourceInvalid { source: String, errors: String },
    #[fail(display = "Failed to create shader, errors: \n{}.", _0)]
    ShaderInvalid(String),
    #[fail(display = "{} is invalid.", _0)]
    HandleInvalid(String),
    #[fail(display = "Out of bounds.")]
    OutOfBounds,
    #[fail(display = "Can NOT update immutable buffer.")]
    UpdateImmutableBuffer,
    #[fail(display = "Can NOT sample render buffer.")]
    SampleRenderBuffer,
    #[fail(display = "Attribute {} is undefined.", _0)]
    AttributeUndefined(String),
    #[fail(display = "Uniform {} is undefined.", _0)]
    UniformUndefined(String),
    #[fail(display = "Failed to create surface, errors:\n{}\n", _0)]
    SurfaceInvalid(String),
    #[fail(display = "Failed to draw, errors:\n{}\n", _0)]
    Draw(String),
    #[fail(display = "OpenGL implementation doesn\'t support {}.", _0)]
    Requirement(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;
