use std::ptr;
use std::str;
use std::ops::Deref;
use std::borrow::Borrow;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum _VariantStr {
    Size8(u8, [u8; 6]),
    Size16(u8, [u8; 14]),
    Size32(u8, [u8; 30]),
    Unconstraint(String),
}

/// UTF-8 encoded owned str with varient length. It will store short string in place
/// instead of another heap space.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantStr(_VariantStr);

impl<T> From<T> for VariantStr
    where T: Borrow<str>
{
    fn from(v: T) -> Self {
        let v = v.borrow();
        unsafe {
            if v.len() <= 6 {
                let mut dst = [0; 6];
                ptr::copy_nonoverlapping(v.as_ptr(), dst.as_mut_ptr(), v.len());
                VariantStr(_VariantStr::Size8(v.len() as u8, dst))
            } else if v.len() <= 14 {
                let mut dst = [0; 14];
                ptr::copy_nonoverlapping(v.as_ptr(), dst.as_mut_ptr(), v.len());
                VariantStr(_VariantStr::Size16(v.len() as u8, dst))
            } else if v.len() <= 30 {
                let mut dst = [0; 30];
                ptr::copy_nonoverlapping(v.as_ptr(), dst.as_mut_ptr(), v.len());
                VariantStr(_VariantStr::Size32(v.len() as u8, dst))
            } else {
                VariantStr(_VariantStr::Unconstraint(String::from(v)))
            }
        }
    }
}

impl Deref for VariantStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Default for VariantStr {
    fn default() -> Self {
        VariantStr(_VariantStr::Size8(0, [0; 6]))
    }
}

impl VariantStr {
    pub fn len(&self) -> usize {
        match self.0 {
            _VariantStr::Size8(len, _) => len as usize,
            _VariantStr::Size16(len, _) => len as usize,
            _VariantStr::Size32(len, _) => len as usize,
            _VariantStr::Unconstraint(ref v) => v.len(),
        }
    }

    /// Converts a slice of bytes to a string slice without checking that the
    /// string contains valid UTF-8.
    pub fn as_str(&self) -> &str {
        unsafe {
            match self.0 {
                _VariantStr::Size8(len, ref v) => str::from_utf8_unchecked(&v[0..len as usize]),
                _VariantStr::Size16(len, ref v) => str::from_utf8_unchecked(&v[0..len as usize]),
                _VariantStr::Size32(len, ref v) => str::from_utf8_unchecked(&v[0..len as usize]),
                _VariantStr::Unconstraint(ref v) => v.as_str(),
            }
        }
    }

    /// Returns a byte slice of this `VariantStr`'s contents.
    pub fn as_bytes(&self) -> &[u8] {
        match self.0 {
            _VariantStr::Size8(len, ref v) => &v[0..len as usize],
            _VariantStr::Size16(len, ref v) => &v[0..len as usize],
            _VariantStr::Size32(len, ref v) => &v[0..len as usize],
            _VariantStr::Unconstraint(ref v) => v.as_bytes(),
        }
    }
}

const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

pub struct VariantChar {
    buf: [u8; 4],
    pos: usize,
}

impl From<char> for VariantChar {
    fn from(c: char) -> Self {
        let code = c as u32;
        let mut buf = [0; 4];
        let pos = if code < MAX_ONE_B {
            buf[3] = code as u8;
            3
        } else if code < MAX_TWO_B {
            buf[2] = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
            buf[3] = (code & 0x3F) as u8 | TAG_CONT;
            2
        } else if code < MAX_THREE_B {
            buf[1] = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
            buf[2] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            buf[3] = (code & 0x3F) as u8 | TAG_CONT;
            1
        } else {
            buf[0] = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
            buf[1] = (code >> 12 & 0x3F) as u8 | TAG_CONT;
            buf[2] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            buf[3] = (code & 0x3F) as u8 | TAG_CONT;
            0
        };
        VariantChar {
            buf: buf,
            pos: pos,
        }
    }
}

impl VariantChar {
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[self.pos..]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn variant_str() {
        let bytes = [49, 0, 0, 0, 0, 0];
        let v = VariantStr::from("1");
        assert_eq!(v.len(), 1);
        assert_eq!(v, VariantStr(_VariantStr::Size8(1, bytes)));

        let bytes = [49, 50, 51, 52, 53, 54];
        let v = VariantStr::from("123456");
        assert_eq!(v.len(), 6);
        assert_eq!(v, VariantStr(_VariantStr::Size8(6, bytes)));

        let bytes = [49, 50, 51, 52, 53, 54, 55, 56, 57, 0, 0, 0, 0, 0];
        let v = VariantStr::from("123456789");
        assert_eq!(v.len(), 9);
        assert_eq!(v, VariantStr(_VariantStr::Size16(9, bytes)));
        assert_eq!(v.as_str(), "123456789");
        assert_eq!(v.as_bytes(), &bytes[0..9]);
    }

    #[test]
    fn layout() {
        assert_eq!(::std::mem::size_of::<VariantStr>(), 32);
    }
}