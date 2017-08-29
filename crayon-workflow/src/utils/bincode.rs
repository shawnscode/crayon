use std::io::{Write, Read};
use std::path::Path;
use std::fs;

use bincode;
use serde;

use errors::*;

pub fn serialize<T, P>(value: &T, path: P) -> Result<()>
    where T: serde::Serialize,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())?;

    let serialization = bincode::serialize(&value, bincode::Infinite)?;
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
    let mut serialization = Vec::new();
    file.read_to_end(&mut serialization)?;

    Ok(bincode::deserialize(&serialization)?)
}