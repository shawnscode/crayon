use crayon_workflow;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
    }

    links {
        Workflow(crayon_workflow::errors::Error, crayon_workflow::errors::ErrorKind);
    }
}