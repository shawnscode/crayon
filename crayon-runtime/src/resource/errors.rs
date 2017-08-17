use std::io;
use image;
use graphics;
use bincode;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(io::Error);
        Image(image::ImageError);
        Bincode(bincode::Error);
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
    }

    errors {
        ResourceDeclarationMismath
        NotFound
    }
}