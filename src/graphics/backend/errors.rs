error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
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
            description("failed compile shader")
            display("Failed compile shader: '{}'", t)
        }
        Unknown
    }
}