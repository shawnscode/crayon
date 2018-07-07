use video::assets::mesh::MeshHandle;
use video::assets::shader::ShaderHandle;
use video::assets::surface::SurfaceHandle;
use video::assets::texture::{RenderTextureHandle, TextureHandle};

use super::backend;
use glutin;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Glutin: {}", _0)]
    Glutin(String),
    #[fail(display = "Backend: {}", _0)]
    Backend(String),
    #[fail(display = "OpenGL implementation doesn\'t support {}.", _0)]
    Requirement(String),
    #[fail(display = "{} is invalid.", _0)]
    SurfaceHandleInvalid(SurfaceHandle),
    #[fail(display = "{} is invalid.", _0)]
    TextureHandleInvalid(TextureHandle),
    #[fail(display = "{} is invalid.", _0)]
    RenderTextureHandleInvalid(RenderTextureHandle),
    #[fail(display = "{} is invalid.", _0)]
    ShaderHandleInvalid(ShaderHandle),
    #[fail(display = "{} is invalid.", _0)]
    MeshHandleInvalid(MeshHandle),
    #[fail(display = "Can not parse attribute from str \'{}\'", _0)]
    AttributeParseFailure(String),
    #[fail(display = "Failed to create shader, errors: \n{}.", _0)]
    ShaderCreationFailure(String),
    #[fail(display = "Failed to submit draw call, errors: \n{}.", _0)]
    DrawFailure(String),
    #[fail(display = "Too many color attachments.")]
    TooManyColorAttachments,
    #[fail(display = "Trying to update immutable buffer.")]
    UpdateImmutableBuffer,
    #[fail(display = "Remote object must be immutable.")]
    CreateMutableRemoteObject,
    #[fail(display = "Shared object must be immutable.")]
    CreateMutableSharedObject,
    #[fail(display = "Window not exists.")]
    WindowNotExists,
    #[fail(display = "Out of bounds.")]
    OutOfBounds,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<backend::errors::Error> for Error {
    fn from(err: backend::errors::Error) -> Error {
        Error::Backend(format!("{}", err))
    }
}

impl From<glutin::CreationError> for Error {
    fn from(err: glutin::CreationError) -> Error {
        Error::Glutin(format!("{}", err))
    }
}

impl From<glutin::ContextError> for Error {
    fn from(err: glutin::ContextError) -> Error {
        Error::Glutin(format!("{}", err))
    }
}
