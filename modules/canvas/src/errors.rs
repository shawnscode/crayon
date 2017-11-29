use crayon::{graphics, resource};
use assets;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
        Font(assets::font_error::Error, assets::font_error::ErrorKind);
    }

    errors {
        NonTransformFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
    }
}