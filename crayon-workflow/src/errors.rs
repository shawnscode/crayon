use bincode;
use std::boxed;
use shader_compiler;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
        Toml(::toml::de::Error);
        Yaml(::serde_yaml::Error);
        Bincode(boxed::Box<bincode::ErrorKind>);
        Image(::image::ImageError);
        Json(::serde_json::Error);
    }

    links {
        Shader(shader_compiler::errors::Error, shader_compiler::errors::ErrorKind);
    }

    errors {
        FileNotFound
        ValidationFailed
        UuidDuplicationFound
        ShaderNotFound
    }
}