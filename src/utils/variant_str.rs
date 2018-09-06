//! UTF-8 encoded owned str with varient length. It will store short string in place
//! instead of another heap space.

use std::borrow::Borrow;
use std::ops::Deref;
use std::{fmt, ptr, str};

#[derive(PartialEq, Eq, Clone)]
enum PrivateVariantStr {
    Stack(u8, [u8; 30]),
    Unconstraint(String),
}

/// UTF-8 encoded owned str with varient length. It will store short string in place
/// instead of another heap space.
#[derive(PartialEq, Eq, Clone)]
pub struct VariantStr(PrivateVariantStr);

impl<T> From<T> for VariantStr
where
    T: Borrow<str>,
{
    fn from(v: T) -> Self {
        let v = v.borrow();
        unsafe {
            if v.len() <= 30 {
                let mut dst = [0; 30];
                ptr::copy_nonoverlapping(v.as_ptr(), dst.as_mut_ptr(), v.len());
                VariantStr(PrivateVariantStr::Stack(v.len() as u8, dst))
            } else {
                VariantStr(PrivateVariantStr::Unconstraint(String::from(v)))
            }
        }
    }
}

impl fmt::Debug for VariantStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            match self.0 {
                PrivateVariantStr::Stack(len, ref v) => {
                    let s = str::from_utf8_unchecked(&v[0..len as usize]);
                    write!(f, "VariantStr::Stack({:?})", s)
                }
                PrivateVariantStr::Unconstraint(ref v) => {
                    write!(f, "VariantStr::Unconstraint({:?})", v.as_str())
                }
            }
        }
    }
}

impl fmt::Display for VariantStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            match self.0 {
                PrivateVariantStr::Stack(len, ref v) => {
                    let s = str::from_utf8_unchecked(&v[0..len as usize]);
                    write!(f, "{}", s)
                }
                PrivateVariantStr::Unconstraint(ref v) => write!(f, "{}", v.as_str()),
            }
        }
    }
}

impl Deref for VariantStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Default for VariantStr {
    #[inline]
    fn default() -> Self {
        VariantStr(PrivateVariantStr::Stack(0, [0; 30]))
    }
}

impl VariantStr {
    /// Gets the len of str.
    #[inline]
    pub fn len(&self) -> usize {
        match self.0 {
            PrivateVariantStr::Stack(len, _) => len as usize,
            PrivateVariantStr::Unconstraint(ref v) => v.len(),
        }
    }

    /// Checks if the `VariantStr` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts a slice of bytes to a string slice without checking that the
    /// string contains valid UTF-8.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            match self.0 {
                PrivateVariantStr::Stack(len, ref v) => {
                    str::from_utf8_unchecked(&v[0..len as usize])
                }
                PrivateVariantStr::Unconstraint(ref v) => v.as_str(),
            }
        }
    }

    /// Returns a byte slice of this `VariantStr`'s contents.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match self.0 {
            PrivateVariantStr::Stack(len, ref v) => &v[0..len as usize],
            PrivateVariantStr::Unconstraint(ref v) => v.as_bytes(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut bytes = [0; 30];
        bytes[0] = 49;
        let v = VariantStr::from("1");
        assert_eq!(v.len(), 1);
        assert_eq!(v, VariantStr(PrivateVariantStr::Stack(1, bytes)));

        bytes[1] = 50;
        bytes[2] = 51;
        bytes[3] = 52;
        bytes[4] = 53;
        bytes[5] = 54;
        let v = VariantStr::from("123456");
        assert_eq!(v.len(), 6);
        assert_eq!(v, VariantStr(PrivateVariantStr::Stack(6, bytes)));
        assert_eq!(v.as_str(), "123456");
        assert_eq!(v.as_bytes(), &bytes[0..6]);
    }

    #[test]
    fn layout() {
        assert_eq!(::std::mem::size_of::<VariantStr>(), 32);
    }
}
