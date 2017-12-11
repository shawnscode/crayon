use glutin;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Backend(super::backend::errors::Error, super::backend::errors::ErrorKind);
    }

    foreign_links {
        Context(glutin::ContextError);
        Creation(glutin::CreationError);
    }

    errors {
        InvalidHandle
        WindowNotExist
        CanNotDrawWithoutView
        CanNotDrawWithoutPipelineState
        CanNotDrawWihtoutVertexBuffer
    }
}