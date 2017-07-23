use graphics;
use super::window;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Window(window::Error, window::ErrorKind);
    }
}