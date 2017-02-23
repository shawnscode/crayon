use glutin;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Context(glutin::ContextError);
    }

    errors {
        InvalidEnum
        InvalidValue
        InvalidOperation
        InvalidFramebufferOperation
        InvalidHandle
        InvalidUpdateStaticResource
        DuplicatedHandle
        OutOfBounds
        FailedCompilePipeline(t: String) {
            description("failed compile pipeline")
            display("Failed compile pipeline: '{}'", t)
        }
        ContextLost
        Unknown
    }
}