use std::fmt;

/// A Universally Unique Identifier (UUID).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Uuid {
    bytes: [u8; 16],
}

impl Uuid {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Uuid { bytes: bytes }
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }
}

impl<'a> fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}
