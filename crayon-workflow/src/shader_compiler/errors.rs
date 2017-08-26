use nom;
use std::fmt;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Nom(nom::ErrorKind);
        Fmt(fmt::Error);
    }

    errors {
        NotSupportVertexAttribute
    }
}