use std::io;
use image;
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

    errors {
        ResourceDeclarationMismath
        NotFound
    }
}