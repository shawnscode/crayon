error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        NonTransformFound
        CanNotInverseTransform
        CanNotAttachSelfAsParent
    }
}