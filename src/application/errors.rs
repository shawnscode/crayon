use graphics;
use resource;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }
}
