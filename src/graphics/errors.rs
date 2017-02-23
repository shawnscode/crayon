error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Backend(super::backend::Error, super::backend::ErrorKind);
    }

    errors {
        InvalidHandle
    }
}