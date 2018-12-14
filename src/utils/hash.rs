//! Faster hashing functionalities by ignoring the cryptographically security needs.
//!
//! Currently, the implemention is based on the Fx algorithm which was extracted from
//! the rustc compiler, it might changes in the future.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// A builder for default Fx hashers.
pub type FastBuildHasher = BuildHasherDefault<hasher::FxHasher>;

/// A `HashMap` using a default Fx hasher.
pub type FastHashMap<K, V> = HashMap<K, V, FastBuildHasher>;

/// A `HashSet` using a default Fx hasher.
pub type FastHashSet<V> = HashSet<V, FastBuildHasher>;

/// A convenience function for when you need a quick 64-bit hash.
#[inline]
pub fn hash64<T: Hash + ?Sized>(v: &T) -> u64 {
    let mut state = hasher::FxHasher64::default();
    v.hash(&mut state);
    state.finish()
}

/// A convenience function for when you need a quick 32-bit hash.
#[inline]
pub fn hash32<T: Hash + ?Sized>(v: &T) -> u32 {
    let mut state = hasher::FxHasher32::default();
    v.hash(&mut state);
    state.finish() as u32
}

/// A convenience function for when you need a quick usize hash.
#[inline]
pub fn hash<T: Hash + ?Sized>(v: &T) -> usize {
    let mut state = hasher::FxHasher::default();
    v.hash(&mut state);
    state.finish() as usize
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut v: HashMap<&'static str, i32> = Default::default();
        v.insert("hahah", 123);
    }
}

mod hasher {
    use std::hash::Hasher;
    use std::ops::BitXor;

    const ROTATE: u32 = 5;
    const SEED64: u64 = 0x517c_c1b7_2722_0a95;
    const SEED32: u32 = (SEED64 & 0xFFFF_FFFF) as u32;

    #[cfg(target_pointer_width = "32")]
    const SEED: usize = SEED32 as usize;
    #[cfg(target_pointer_width = "64")]
    const SEED: usize = SEED64 as usize;

    trait HashWord {
        fn hash_word(&mut self, word: Self);
    }

    macro_rules! impl_hash_word {
        ($($ty:ty = $key:ident),* $(,)*) => (
            $(
                impl HashWord for $ty {
                    #[inline]
                    fn hash_word(&mut self, word: Self) {
                        *self = self.rotate_left(ROTATE).bitxor(word).wrapping_mul($key);
                    }
                }
            )*
        )
    }

    impl_hash_word!(usize = SEED, u32 = SEED32, u64 = SEED64);

    macro_rules! read_num_bytes {
        ($ty:ty, $size:expr, $src:expr, $which:ident) => {{
            assert!($size == ::std::mem::size_of::<$ty>());
            assert!($size <= $src.len());
            let mut data: $ty = 0;
            unsafe {
                ::std::ptr::copy_nonoverlapping(
                    $src.as_ptr(),
                    &mut data as *mut $ty as *mut u8,
                    $size,
                );
            }
            data.$which()
        }};
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn read_u32(bytes: &[u8]) -> u32 {
        read_num_bytes!(u32, 4, bytes, to_be)
    }

    #[cfg(target_endian = "little")]
    #[inline]
    fn read_u32(bytes: &[u8]) -> u32 {
        read_num_bytes!(u32, 4, bytes, to_le)
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn read_u64(bytes: &[u8]) -> u64 {
        read_num_bytes!(u64, 8, bytes, to_be)
    }

    #[cfg(target_endian = "little")]
    #[inline]
    fn read_u64(bytes: &[u8]) -> u64 {
        read_num_bytes!(u64, 8, bytes, to_le)
    }

    #[inline]
    fn write32(mut hash: u32, mut bytes: &[u8]) -> u32 {
        while bytes.len() >= 4 {
            let n = read_u32(bytes);
            hash.hash_word(n);
            bytes = bytes.split_at(4).1;
        }

        for byte in bytes {
            hash.hash_word(u32::from(*byte));
        }
        hash
    }

    #[inline]
    fn write64(mut hash: u64, mut bytes: &[u8]) -> u64 {
        while bytes.len() >= 8 {
            let n = read_u64(bytes);
            hash.hash_word(n);
            bytes = bytes.split_at(8).1;
        }

        if bytes.len() >= 4 {
            let n = read_u32(bytes);
            hash.hash_word(u64::from(n));
            bytes = bytes.split_at(4).1;
        }

        for byte in bytes {
            hash.hash_word(u64::from(*byte));
        }
        hash
    }

    #[inline]
    #[cfg(target_pointer_width = "32")]
    fn write(hash: usize, bytes: &[u8]) -> usize {
        write32(hash as u32, bytes) as usize
    }

    #[inline]
    #[cfg(target_pointer_width = "64")]
    fn write(hash: usize, bytes: &[u8]) -> usize {
        write64(hash as u64, bytes) as usize
    }

    /// This hashing algorithm was extracted from the Rustc compiler.
    /// This is the same hashing algoirthm used for some internal operations in FireFox.
    /// The strength of this algorithm is in hashing 8 bytes at a time on 64-bit platforms,
    /// where the FNV algorithm works on one byte at a time.
    ///
    /// This hashing algorithm should not be used for cryptographic, or in scenarios where
    /// DOS attacks are a concern.
    #[derive(Debug, Clone)]
    pub struct FxHasher {
        hash: usize,
    }

    impl Default for FxHasher {
        #[inline]
        fn default() -> FxHasher {
            FxHasher { hash: 0 }
        }
    }

    impl Hasher for FxHasher {
        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            self.hash = write(self.hash, bytes);
        }

        #[inline]
        fn write_u8(&mut self, i: u8) {
            self.hash.hash_word(i as usize);
        }

        #[inline]
        fn write_u16(&mut self, i: u16) {
            self.hash.hash_word(i as usize);
        }

        #[inline]
        fn write_u32(&mut self, i: u32) {
            self.hash.hash_word(i as usize);
        }

        #[inline]
        #[cfg(target_pointer_width = "32")]
        fn write_u64(&mut self, i: u64) {
            self.hash.hash_word(i as usize);
            self.hash.hash_word((i >> 32) as usize);
        }

        #[inline]
        #[cfg(target_pointer_width = "64")]
        fn write_u64(&mut self, i: u64) {
            self.hash.hash_word(i as usize);
        }

        #[inline]
        fn write_usize(&mut self, i: usize) {
            self.hash.hash_word(i);
        }

        #[inline]
        fn finish(&self) -> u64 {
            self.hash as u64
        }
    }

    /// This hashing algorithm was extracted from the Rustc compiler.
    ///
    /// This is the same hashing algoirthm used for some internal operations in FireFox.
    /// The strength of this algorithm is in hashing 4 bytes at a time on any platform,
    /// where the FNV algorithm works on one byte at a time.
    ///
    /// This hashing algorithm should not be used for cryptographic, or in scenarios where
    /// DOS attacks are a concern.
    #[derive(Debug, Clone)]
    pub struct FxHasher32 {
        hash: u32,
    }

    impl Default for FxHasher32 {
        #[inline]
        fn default() -> FxHasher32 {
            FxHasher32 { hash: 0 }
        }
    }

    impl Hasher for FxHasher32 {
        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            self.hash = write32(self.hash, bytes);
        }

        #[inline]
        fn write_u8(&mut self, i: u8) {
            self.hash.hash_word(u32::from(i));
        }

        #[inline]
        fn write_u16(&mut self, i: u16) {
            self.hash.hash_word(u32::from(i));
        }

        #[inline]
        fn write_u32(&mut self, i: u32) {
            self.hash.hash_word(i);
        }

        #[inline]
        fn write_u64(&mut self, i: u64) {
            self.hash.hash_word(i as u32);
            self.hash.hash_word((i >> 32) as u32);
        }

        #[inline]
        #[cfg(target_pointer_width = "32")]
        fn write_usize(&mut self, i: usize) {
            self.write_u32(i as u32);
        }

        #[inline]
        #[cfg(target_pointer_width = "64")]
        fn write_usize(&mut self, i: usize) {
            self.write_u64(i as u64);
        }

        #[inline]
        fn finish(&self) -> u64 {
            u64::from(self.hash)
        }
    }

    /// This hashing algorithm was extracted from the Rustc compiler.
    ///
    /// This is the same hashing algoirthm used for some internal operations in FireFox.
    /// The strength of this algorithm is in hashing 8 bytes at a time on any platform,
    /// where the FNV algorithm works on one byte at a time.
    ///
    /// This hashing algorithm should not be used for cryptographic, or in scenarios where
    /// DOS attacks are a concern.
    #[derive(Debug, Clone)]
    pub struct FxHasher64 {
        hash: u64,
    }

    impl Default for FxHasher64 {
        #[inline]
        fn default() -> FxHasher64 {
            FxHasher64 { hash: 0 }
        }
    }

    impl Hasher for FxHasher64 {
        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            self.hash = write64(self.hash, bytes);
        }

        #[inline]
        fn write_u8(&mut self, i: u8) {
            self.hash.hash_word(u64::from(i));
        }

        #[inline]
        fn write_u16(&mut self, i: u16) {
            self.hash.hash_word(u64::from(i));
        }

        #[inline]
        fn write_u32(&mut self, i: u32) {
            self.hash.hash_word(u64::from(i));
        }

        fn write_u64(&mut self, i: u64) {
            self.hash.hash_word(i);
        }

        #[inline]
        fn write_usize(&mut self, i: usize) {
            self.hash.hash_word(i as u64);
        }

        #[inline]
        fn finish(&self) -> u64 {
            self.hash
        }
    }
}
