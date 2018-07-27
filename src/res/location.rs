use std::path::{Path, PathBuf};

use utils::hash_value::HashValue;
use utils::uuid::Uuid;

use super::errors::*;

#[derive(Debug)]
pub enum Location {
    Uuid(Uuid),
    Str(HashValue<str>, PathBuf),
}

impl Location {
    pub fn from_str(v: &str) -> Result<Location> {
        let idx = v.find(':').ok_or_else(|| Error::MalformLocation(v.into()))?;
        let (fs, file) = v.split_at(idx);
        let path: &Path = (&file[1..]).as_ref();
        Ok(Location::Str(fs.into(), path.into()))
    }

    pub fn from(v: Uuid) -> Location {
        Location::Uuid(v)
    }
}
