use graphics;
use resource;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }

    errors {
        NonTransformFound
        NonCameraFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
        CanNotDrawWithoutCamera
        MissingRequiredComponent(t: String) {
            description("missing required component(s).")
            display("Missing required component(s): '{}'", t) 
        }
    }
}