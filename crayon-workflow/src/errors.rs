error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(::std::io::Error);
        Toml(::toml::de::Error);
        Yaml(::serde_yaml::Error);
        Image(::image::ImageError);
    }
}