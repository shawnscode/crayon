use std::path::Path;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher};

use utils::hash;

#[derive(Debug, Eq)]
pub struct HashValue<T>(u64, PhantomData<T>)
where
    T: Hash + ?Sized;

impl<T> HashValue<T>
where
    T: Hash + ?Sized,
{
    pub fn zero() -> Self {
        HashValue(0, PhantomData)
    }
}

impl<T> Clone for HashValue<T>
where
    T: Hash + ?Sized,
{
    fn clone(&self) -> Self {
        HashValue(self.0, self.1)
    }
}

impl<T> Copy for HashValue<T>
where
    T: Hash + ?Sized,
{
}

impl<T> Hash for HashValue<T>
where
    T: Hash + ?Sized,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
    }
}

impl<T> PartialEq<HashValue<T>> for HashValue<T>
where
    T: Hash + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<F> From<F> for HashValue<str>
where
    F: AsRef<str>,
{
    fn from(v: F) -> Self {
        HashValue(hash(&v.as_ref()), PhantomData)
    }
}

impl<T> PartialEq<T> for HashValue<str>
where
    T: AsRef<str>,
{
    fn eq(&self, rhs: &T) -> bool {
        hash(&rhs.as_ref()) == self.0
    }
}

impl<T> From<T> for HashValue<Path>
where
    T: AsRef<Path>,
{
    fn from(v: T) -> Self {
        HashValue(hash(&v.as_ref()), PhantomData)
    }
}

impl<T> PartialEq<T> for HashValue<Path>
where
    T: AsRef<Path>,
{
    fn eq(&self, rhs: &T) -> bool {
        hash(&rhs.as_ref()) == self.0
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn hash_str() {
        assert_eq!(HashValue::<str>::from("hash_str"), "hash_str");
    }

    #[test]
    fn collections() {
        let mut set = HashSet::<HashValue<str>>::new();
        set.insert(HashValue::from("asdasd"));
        set.insert(HashValue::from("asdasd"));
        set.insert(HashValue::from("asdasd"));
        set.insert(HashValue::from("asdasd"));
        assert_eq!(set.len(), 1);
        assert_eq!(
            set.get(&("asdasd".into())),
            Some(&HashValue::from("asdasd"))
        );
    }

    #[test]
    fn hash_path() {
        let h = HashValue::<Path>::from("str_path");
        let _ = HashValue::<Path>::from(h);
    }
}
