#[derive(Debug, Fail)]
pub enum Error {
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
    #[fail(display = "Failed to create surface, errors:\n{}\n", _0)]
    SurfaceInvalid(String),
    #[fail(display = "Attribute({}) is undefined.", _0)]
    AttributeUndefined(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;
