use crayon::ecs::prelude::*;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Ent({:?}) does not have a node.", _0)]
    NonNodeFound(Entity),
    #[fail(display = "The transform of ent({:?}) can not be inversed.", _0)]
    CanNotInverseTransform(Entity),
    #[fail(display = "Node can not set self as parent.")]
    CanNotAttachSelfAsParent,
}

pub type Result<T> = ::std::result::Result<T, Error>;
