use nom;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Nom(nom::ErrorKind);
    }

    errors {
        NotSupportVertexAttribute
    }
}