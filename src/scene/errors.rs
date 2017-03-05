use graphics;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
    }

    errors {
        NonTransformFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
        CanNotDrawWithoutCamera
        MissingRequiredComponent(t: String) {
            description("missing required component(s).")
            display("Missing required component(s): '{}'", t) 
        }
    }
}