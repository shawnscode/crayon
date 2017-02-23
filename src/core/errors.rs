use glutin;
use graphics;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
    }

    foreign_links {
        Context(glutin::ContextError);
        Creation(glutin::CreationError);
    }

    errors {
        ContextLost
    }
}