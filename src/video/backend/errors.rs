#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An unacceptable value is specified for an enumerated argument.")]
    GLEnumInvalid,
    #[fail(display = "A numeric argument is out of range.")]
    GLValueInvalid,
    #[fail(display = "The specified operation is not allowed in the current state.")]
    GLOperationInvalid,
    #[fail(
        display = "The command is trying to render to or read from the framebuffer
                        while the currently bound framebuffer is not framebuffer 
                        complete."
    )]
    GLFrameBufferOperationInvalid,
    #[fail(display = "There is not enough memory left to execute the command.")]
    GLOutOfMemory,
    #[fail(display = "Failed to compile shader, errors:\n{}\nsource:\n{}\n", errors, source)]
    GLShaderCompileFailure { source: String, errors: String },
    #[fail(display = "FrameBuffer is incomplete.")]
    GLFrameBufferIncomplete,
    #[fail(display = "Failed to compile pipeline, errors:\n{}\n", _0)]
    GLPipelineCompileFailure(String),
    #[fail(display = "Failed to get string, errors:\n{}\n", _0)]
    GLGetStrFailure(String),
    #[fail(display = "Unknown OpenGL error.")]
    GLUnknown,
    #[fail(display = "Handle is invalid.")]
    HandleInvalid,
    #[fail(display = "Handle is duplicated.")]
    HandleDuplicated,
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
    SurfaceCreationFailure(String),
    #[fail(display = "Failed to draw, errors:\n{}\n", _0)]
    DrawFailure(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;
