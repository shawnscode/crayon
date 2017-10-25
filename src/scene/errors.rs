error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        NonTransformFound
        NonCameraFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
        CanNotDrawWithoutCamera
    }
}