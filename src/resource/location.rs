use std::path::Path;
use utils::HashValue;

/// A `Location` describes where the source data for a resource is located. It
/// usually contains a Path that can be resolved to an URI. `Location`s are also
/// used for sharing. If 2 `Location`s are completely identical, they identify
/// the same resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location<'a> {
    code: Signature,
    location: &'a Path,
}

impl<'a> Location<'a> {
    pub fn unique<P>(location: &'a P) -> Self
        where P: ?Sized + AsRef<Path>
    {
        Location {
            location: location.as_ref(),
            code: Signature::Unique,
        }
    }

    pub fn shared<P>(code: u8, location: &'a P) -> Self
        where P: ?Sized + AsRef<Path>
    {
        Location {
            location: location.as_ref(),
            code: Signature::Shared(code),
        }
    }

    /// Returns true if this location is shared.
    pub fn is_shared(&self) -> bool {
        self.code != Signature::Unique
    }

    /// Gets the uniform resource identifier.
    pub fn uri(&self) -> &Path {
        &self.location
    }

    /// Gets hash object of `Location`.
    pub fn hash(&self) -> LocationAtom {
        LocationAtom::from(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash)]
enum Signature {
    Unique,
    Shared(u8),
}

impl PartialEq<Signature> for Signature {
    fn eq(&self, other: &Signature) -> bool {
        match *self {
            Signature::Unique => false,
            Signature::Shared(lhs) => {
                match *other {
                    Signature::Shared(rhs) => lhs == rhs,
                    _ => false,
                }
            }
        }
    }
}

/// Hash object of `Location`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationAtom {
    code: Signature,
    location: HashValue<Path>,
}

impl LocationAtom {
    pub fn from(v: &Location) -> Self {
        LocationAtom {
            code: v.code,
            location: v.location.into(),
        }
    }

    /// Returns true if this location is shared.
    pub fn is_shared(&self) -> bool {
        self.code != Signature::Unique
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn basic() {
        let l1 = Location::unique("1");
        let l2 = Location::unique("1");
        assert!(l1 != l2);

        let l1 = Location::shared(0, "1");
        let l2 = Location::shared(1, "1");
        assert!(l1 != l2);

        let l1 = Location::shared(0, "1");
        let l2 = Location::shared(0, "1");
        assert!(l1 == l2);
    }

    #[test]
    fn shared_container() {
        let l1 = Location::shared(0, "1").hash();
        let l2 = Location::shared(0, "2").hash();

        let mut map = HashSet::new();
        assert_eq!(map.insert(l1), true);
        assert_eq!(map.contains(&l1), true);
        assert_eq!(map.contains(&l2), false);

        assert_eq!(map.insert(l2), true);
        assert_eq!(map.contains(&l1), true);
        assert_eq!(map.contains(&l2), true);

        assert_eq!(map.insert(l1), false);
    }

    #[test]
    fn unique_container() {
        let l1 = Location::unique("1").hash();

        let mut map = HashSet::new();
        assert_eq!(map.insert(l1), true);
        assert_eq!(map.contains(&l1), false);
    }
}