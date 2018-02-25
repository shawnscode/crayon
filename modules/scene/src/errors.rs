use crayon::{graphics, resource};

use assets::pipeline::PipelineHandle;
use assets::material::MaterialHandle;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)] Graphics(graphics::errors::Error),
    #[fail(display = "{}", _0)] Resource(resource::errors::Error),
    #[fail(display = "{} is invalid.", _0)] PipelineHandleInvalid(PipelineHandle),
    #[fail(display = "{} is invalid.", _0)] MaterialHandleInvalid(MaterialHandle),
    #[fail(display = "No node found.")] NonNodeFound,
    #[fail(display = "No transform found.")] NonTransformFound,
    #[fail(display = "No camera found.")] NonCameraFound,
    #[fail(display = "The transform can not be inversed.")] CanNotInverseTransform,
    #[fail(display = "Node can not set self as parent.")] CanNotAttachSelfAsParent,
    #[fail(display = "Uniform decleartion mismatch.")] UniformMismatch,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<graphics::errors::Error> for Error {
    fn from(err: graphics::errors::Error) -> Self {
        Error::Graphics(err)
    }
}

impl From<resource::errors::Error> for Error {
    fn from(err: resource::errors::Error) -> Self {
        Error::Resource(err)
    }
}
