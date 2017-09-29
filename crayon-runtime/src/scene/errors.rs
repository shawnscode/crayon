use graphics;
use resource;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        GraphicsFrontend(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }

    errors {
        NonTransformFound
        NonCameraFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
        CanNotDrawWithoutCamera
    }
}