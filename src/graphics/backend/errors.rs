use core::window;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Window(window::Error, window::ErrorKind);
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
        Unknown
    }
}