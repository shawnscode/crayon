use std::fs;
use std::path::Path;
use std::io::{Write, Read};

use serde;
use serde_yaml;
use bincode;

use errors::*;

pub fn serialize<T, P>(value: &T, path: P, readable: bool) -> Result<()>
    where T: serde::Serialize,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())?;

    if readable {
        let raw = serde_yaml::to_string(&value)?;
        file.write(raw.as_ref())?;
    } else {
        let raw = bincode::serialize(&value, bincode::Infinite)?;
        file.write(raw.as_ref())?;
    };

    file.flush()?;
    Ok(())
}

pub fn deserialize<T, P>(path: P, readable: bool) -> Result<T>
    where T: serde::de::DeserializeOwned,
          P: AsRef<Path>
{
    let path = path.as_ref();
    if path.exists() {
        let mut file = fs::OpenOptions::new().read(true).open(path)?;

        if readable {
            let mut raw = String::new();
            file.read_to_string(&mut raw)?;
            Ok(serde_yaml::from_str(&raw)?)
        } else {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok(bincode::deserialize(&bytes)?)
        }
    } else {
        bail!("failed to deserilize from path {:?}.", path);
    }
}