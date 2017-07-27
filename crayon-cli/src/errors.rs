use crayon_workflow;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
    }

    links {
        Workflow(crayon_workflow::Error, crayon_workflow::ErrorKind);
    }
}