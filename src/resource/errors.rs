use std::io;
use image;
use graphics;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(io::Error);
        Image(image::ImageError);
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
    }

    errors {
        ResourceDeclarationMismath
    }
}