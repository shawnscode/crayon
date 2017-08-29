use std::io::{Write, Read};
use std::path::Path;
use std::fs;

use toml;
use serde;

use errors::*;

pub fn load<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
    let mut next = value;
    for i in path {
        if let Some(leaf) = next.get(i) {
            next = leaf;
        } else {
            return None;
        }
    }
    Some(next)
}

pub fn load_as_i32(value: &toml::Value, path: &[&str]) -> Option<i32> {
    if path.len() <= 0 {
        None
    } else {
        let parent = if path.len() > 1 {
            load(&value, &path[..path.len() - 1])
        } else {
            Some(value)
        };

        parent
            .and_then(|v| v.get(path[path.len() - 1]))
            .and_then(|v| v.as_integer())
            .map(|v| v as i32)
    }
}

pub fn load_as_u32(value: &toml::Value, path: &[&str]) -> Option<u32> {
    if path.len() <= 0 {
        None
    } else {
        let parent = if path.len() > 1 {
            load(&value, &path[..path.len() - 1])
        } else {
            Some(value)
        };

        parent
            .and_then(|v| v.get(path[path.len() - 1]))
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
    }
}

pub fn load_as_str<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a str> {
    if path.len() <= 0 {
        None
    } else {
        let parent = if path.len() > 1 {
            load(&value, &path[..path.len() - 1])
        } else {
            Some(value)
        };

        parent
            .and_then(|v| v.get(path[path.len() - 1]))
            .and_then(|v| v.as_str())
    }
}

pub fn load_as_array<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a Vec<toml::Value>> {
    if path.len() <= 0 {
        None
    } else {
        let parent = if path.len() > 1 {
            load(&value, &path[..path.len() - 1])
        } else {
            Some(value)
        };

        parent
            .and_then(|v| v.get(path[path.len() - 1]))
            .and_then(|v| v.as_array())
    }
}

pub fn serialize<T, P>(value: &T, path: P) -> Result<()>
    where T: serde::Serialize,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())?;

    let serialization = toml::to_string(&value)?;
    file.write(serialization.as_ref())?;
    file.flush()?;

    Ok(())
}

pub fn deserialize<T, P>(path: P) -> Result<T>
    where T: serde::de::DeserializeOwned,
          P: AsRef<Path>
{
    let path = path.as_ref();
    if !path.exists() {
        bail!(ErrorKind::FileNotFound);
    }

    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut serialization = String::new();
    file.read_to_string(&mut serialization)?;

    Ok(toml::from_str(&serialization)?)
}