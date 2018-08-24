use std::path::{Path, PathBuf};
use uuid::Uuid;

use errors::*;
use utils::hash_value::HashValue;

#[derive(Debug)]
pub enum Location {
    Uuid(Uuid),
    Name(HashValue<str>, PathBuf),
}

impl Location {
    pub fn from_str(v: &str) -> Result<Location> {
        if let Some(idx) = v.find(':') {
            let (fs, file) = v.split_at(idx);
            let path: &Path = (&file[1..]).as_ref();
            Ok(Location::Name(fs.into(), path.into()))
        } else {
            bail!("Malformated location {:?}.", v);
        }
    }

    pub fn from(v: Uuid) -> Location {
        Location::Uuid(v)
    }
}
