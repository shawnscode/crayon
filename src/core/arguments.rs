use std::path::{Path, Component};
use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};
use json;

///
pub struct Arguments {
    value: json::JsonValue,
}

impl Arguments {
    pub fn new<T>(path: T) -> Result<Arguments>
        where T: AsRef<Path>
    {
        let mut buf = String::new();
        let mut file = File::open(&path)?;
        file.read_to_string(&mut buf)?;

        let value = json::parse(&buf).map_err(|err| {
                match err {
                    json::Error::UnexpectedCharacter { ch, line, column } => {
                        let msg = format!("Unexpected character {} at line {} column {}.",
                                          ch,
                                          line,
                                          column);
                        Error::new(ErrorKind::InvalidInput, msg)
                    }
                    json::Error::UnexpectedEndOfJson => {
                        Error::new(ErrorKind::InvalidData, "Unexpected end of json.")
                    }
                    json::Error::FailedUtf8Parsing => {
                        Error::new(ErrorKind::InvalidData, "Failed to do utf8 parsing.")
                    }
                    json::Error::ExceededDepthLimit => {
                        Error::new(ErrorKind::InvalidData, "Exceeded depth limit.")
                    }
                    json::Error::WrongType(s) => {
                        let msg = format!("Wrong type {}.", s);
                        Error::new(ErrorKind::InvalidData, msg)
                    }
                }
            })?;

        Ok(Arguments { value: value })
    }

    fn load<T>(&self, path: T) -> Option<&json::JsonValue>
        where T: AsRef<Path>
    {
        let components = path.as_ref().components();

        let mut value = &self.value;
        let mut parent = &self.value;

        for c in components {
            match c {
                Component::ParentDir => {
                    value = &parent;
                }
                Component::Normal(name) => {
                    let name = name.to_str().unwrap();
                    if value.has_key(&name) {
                        parent = &value;
                        value = &value[name];
                    } else {
                        return None;
                    }
                }
                _ => {}
            };
        }

        Some(value)
    }

    pub fn load_as_i32<T>(&self, path: T) -> Option<i32>
        where T: AsRef<Path>
    {
        self.load(path).and_then(|v| v.as_i32())
    }

    pub fn load_as_bool<T>(&self, path: T) -> Option<bool>
        where T: AsRef<Path>
    {
        self.load(path).and_then(|v| v.as_bool())
    }

    pub fn load_as_str<T>(&self, path: T) -> Option<&str>
        where T: AsRef<Path>
    {
        self.load(path).and_then(|v| v.as_str())
    }
}