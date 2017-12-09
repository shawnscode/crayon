use crayon::{graphics, resource};
use assets;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
        Assets(assets::errors::Error, assets::errors::ErrorKind);
    }

    errors {
        NonTransformFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
    }
}