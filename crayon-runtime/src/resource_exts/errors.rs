use std::io;
use zip;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(io::Error);
        Zip(zip::result::ZipError);
    }

    errors {
       DriveWithSameIdentFound
       DriveNotFound
       NotFound
       FutureCanceled
    }
}