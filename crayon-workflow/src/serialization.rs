use std::fs;
use std::path::Path;
use std::io::{Write, Read};

use serde;
use serde_yaml;

use errors::*;

pub fn serialize<T, P>(value: &T, path: P, _readable: bool) -> Result<()>
    where T: serde::Serialize,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())?;

    let raw = serde_yaml::to_string(&value)?;
    file.write(raw.as_ref())?;
    file.flush()?;
    Ok(())
}

pub fn deserialize<T, P>(path: P, _readable: bool) -> Result<T>
    where T: serde::de::DeserializeOwned,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .open(path.as_ref())?;

    let mut raw = String::new();
    file.read_to_string(&mut raw)?;

    Ok(serde_yaml::from_str(&raw)?)
}