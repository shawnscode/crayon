use glutin;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Context(glutin::ContextError);
    }

    errors {
        InvalidEnum
        InvalidValue
        InvalidOperation
        InvalidFramebufferOperation
        OutOfBounds
        Unknown
    }
}