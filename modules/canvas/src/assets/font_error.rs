use crayon::{resource, graphics};

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }

    errors {
        GlyphTooLarge
        NoRoomForWholeQueue
        GlyphNotCached
    }
}