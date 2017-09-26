use bincode;
use std::boxed;
use shaderc;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
        TomlDeser(::toml::de::Error);
        TomlSer(::toml::ser::Error);
        Yaml(::serde_yaml::Error);
        Bincode(boxed::Box<bincode::ErrorKind>);
        Image(::image::ImageError);
        Json(::serde_json::Error);
    }

    links {
        Shader(shaderc::errors::Error, shaderc::errors::ErrorKind);
    }

    errors {
        FileNotFound
        WorkspaceNotFound
        ShaderNotFound
    }
}