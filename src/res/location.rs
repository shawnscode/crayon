use errors::*;
use utils::HashValue;

/// A `Location` describes where the source data for a resource is located. If two
/// `Location`s are completely identical, they identify the same resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location<'a> {
    vfs: &'a str,
    filename: &'a str,
}

impl<'a> Location<'a> {
    pub fn new(location: &'a str) -> Result<Self> {
        let (vfs, filename) = Self::schema(location)?;

        Ok(Location {
            vfs: vfs,
            filename: filename,
        })
    }

    /// Gets the filename.
    #[inline]
    pub fn filename(&self) -> &str {
        self.filename
    }

    /// Gets the identifier of the virtual filesystem.
    #[inline]
    pub fn vfs(&self) -> &str {
        self.vfs
    }

    fn schema(location: &'a str) -> Result<(&'a str, &'a str)> {
        location
            .find(':')
            .map(|index| {
                let (fs, name) = location.split_at(index);
                (fs, &name[1..])
            }).ok_or_else(|| {
                format_err!(
                    "{} does not match location schema [vfs:filename].",
                    location
                )
            })
    }
}

impl<'a> From<&'a str> for Location<'a> {
    fn from(location: &'a str) -> Self {
        Location::new(location).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashValueLocation {
    vfs: HashValue<str>,
    filename: HashValue<str>,
}

impl<'a> From<Location<'a>> for HashValueLocation {
    fn from(location: Location<'a>) -> Self {
        HashValueLocation {
            vfs: location.vfs.into(),
            filename: location.filename.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use utils::FastHashSet;

    #[test]
    fn basic() {
        let loc = Location::new("res:textures/crate.png").unwrap();
        assert_eq!(loc.vfs(), "res");
        assert_eq!(loc.filename(), "textures/crate.png");
        assert_eq!(loc, "res:textures/crate.png".into());

        assert!(Location::new("res:test/1/2/3/s.png").is_ok());
        assert!(Location::new("res").is_err());
        assert!(Location::new("crate.png").is_err());
    }

    #[test]
    fn container() {
        let l1 = Location::new("res:1").unwrap();
        let l2 = Location::new("res:2").unwrap();

        let mut set = FastHashSet::default();
        assert_eq!(set.insert(l1), true);
        assert_eq!(set.contains(&l1), true);
        assert_eq!(set.contains(&l2), false);

        assert_eq!(set.insert(l2), true);
        assert_eq!(set.contains(&l1), true);
        assert_eq!(set.contains(&l2), true);

        assert_eq!(set.insert(l1), false);
    }

    #[test]
    #[should_panic]
    fn from() {
        let _: Location = "res".into();
    }

    #[test]
    fn hash_value() {
        let l1: HashValueLocation = Location::new("res:1").unwrap().into();
        let l2: HashValueLocation = Location::new("res:2").unwrap().into();

        let mut set = FastHashSet::default();
        assert_eq!(set.insert(l1), true);
        assert_eq!(set.contains(&l1), true);
        assert_eq!(set.contains(&l2), false);

        assert_eq!(set.insert(l2), true);
        assert_eq!(set.contains(&l1), true);
        assert_eq!(set.contains(&l2), true);

        assert_eq!(set.insert(l1), false);
    }
}
