use graphics;
use super::window;
use resource;
use scene;
use bincode;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
        Bincode(bincode::Error);
    }

    links {
        GraphicsFrontend(graphics::errors::Error, graphics::errors::ErrorKind);
        Window(window::Error, window::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
        Scene(scene::errors::Error, scene::errors::ErrorKind);
    }
}