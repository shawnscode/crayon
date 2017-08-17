use std::path::{Path, Component};
use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};
use json;

/// Loads configuration into a tree based structure for easy usage.
pub struct Arguments {
    value: json::JsonValue,
}

impl Arguments {
    /// Creates and initialize a `Arguments` with file path. The file should
    /// be a valid utf-8 encoded json.
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

    fn load<T>(base: &json::JsonValue, path: T) -> Option<&json::JsonValue>
        where T: AsRef<Path>
    {
        let components = path.as_ref().components();

        let mut value = base;
        let mut parent = base;

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

    /// Try to load value at specified path as i32.
    pub fn load_as_i32<T>(&self, path: T) -> Option<i32>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_i32())
    }

    /// Try to load value at specified path as boolean.
    pub fn load_as_bool<T>(&self, path: T) -> Option<bool>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_bool())
    }

    /// Try to load value at specified path as str.
    pub fn load_as_str<T>(&self, path: T) -> Option<&str>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_str())
    }

    /// Try to load value at specified path as slice of `Arguments`.
    pub fn load_as_slice<T>(&self, path: T) -> Option<Slice>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| Some(Slice { value: v }))
    }
}

/// An slice of `Arguments`.
pub struct Slice<'a> {
    value: &'a json::JsonValue,
}

impl<'a> Slice<'a> {
    /// Try to load value at specified path as i32.
    pub fn load_as_i32<T>(&self, path: T) -> Option<i32>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_i32())
    }

    /// Try to load value at specified path as boolean.
    pub fn load_as_bool<T>(&self, path: T) -> Option<bool>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_bool())
    }

    /// Try to load value at specified path as str.
    pub fn load_as_str<T>(&self, path: T) -> Option<&str>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| v.as_str())
    }

    /// Try to load value at specified path as slice of `Arguments`.
    pub fn load_as_slice<T>(&self, path: T) -> Option<Slice>
        where T: AsRef<Path>
    {
        Arguments::load(&self.value, path).and_then(|v| Some(Slice { value: v }))
    }
}