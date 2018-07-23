//! Byte order extensions of `std::io::Read and std::io::Write` in `BigEndian`.

use std::io::{Read, Result, Write};
use std::{mem, ptr, slice};

macro_rules! read_num_bytes {
    ($ty:ty, $size:expr, $src:expr, $which:ident) => {{
        assert!($size == mem::size_of::<$ty>());
        assert!($size <= $src.len());

        unsafe {
            let mut data: $ty = 0;
            ptr::copy_nonoverlapping($src.as_ptr(), &mut data as *mut $ty as *mut u8, $size);
            data.$which()
        }
    }};
}

/// Extends `std::io::Read` with methods for reading numbers in `BigEndian`.
pub trait ByteOrderRead: Read {
    /// Reads an unsigned 8 bit integer from the underlying reader.
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Reads an unsigned 16 bit integer from the underlying reader.
    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(read_num_bytes!(u16, 2, &buf, to_be))
    }

    /// Reads an unsigned 32 bit integer from the underlying reader.
    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(read_num_bytes!(u32, 4, &buf, to_be))
    }

    /// Reads an unsigned 64 bit integer from the underlying reader.
    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(read_num_bytes!(u64, 8, &buf, to_be))
    }

    /// Reads an signed 8 bit integer from the underlying reader.
    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().map(|v| v as i8)
    }

    /// Reads a signed 16 bit integer from the underlying reader.
    #[inline]
    fn read_i16(&mut self) -> Result<i16> {
        self.read_u16().map(|v| v as i16)
    }

    /// Reads a signed 32 bit integer from the underlying reader.
    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        self.read_u32().map(|v| v as i32)
    }

    /// Reads a signed 64 bit integer from the underlying reader.
    #[inline]
    fn read_i64(&mut self) -> Result<i64> {
        self.read_u64().map(|v| v as i64)
    }

    /// Reads a IEEE754 single-precision (4 bytes) floating point number from
    /// the underlying reader.
    #[inline]
    fn read_f32(&mut self) -> Result<f32> {
        self.read_u32()
            .map(|v| unsafe { *(&v as *const u32 as *const f32) })
    }

    /// Reads a IEEE754 double-precision (8 bytes) floating point number from
    /// the underlying reader.
    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        self.read_u64()
            .map(|v| unsafe { *(&v as *const u64 as *const f64) })
    }

    /// Reads a copyable value in big endian device.
    ///
    /// N.B.: https://github.com/rust-lang/rust/issues/24111
    #[cfg(target_endian = "big")]
    #[inline]
    fn read<T: Copy + Sized>(&mut self, buf: &mut [u8]) -> Result<T> {
        unsafe {
            assert!(buf.len() >= mem::size_of::<T>());
            let mut slice = &mut buf[0..mem::size_of::<T>()];
            self.read_exact(&mut slice)?;
            Ok(*(slice.as_ptr() as *const _))
        }
    }

    /// Reads a copyable value in little endian device.
    ///
    /// N.B.: https://github.com/rust-lang/rust/issues/24111
    #[cfg(target_endian = "little")]
    #[inline]
    fn read<T: Copy + Sized>(&mut self, buf: &mut [u8]) -> Result<T> {
        unsafe {
            assert!(buf.len() >= mem::size_of::<T>());

            let mut slice = &mut buf[0..mem::size_of::<T>()];
            self.read_exact(&mut slice)?;

            slice.reverse();
            Ok(*(slice.as_ptr() as *const _))
        }
    }
}

/// All types that implement `Read` get methods defined in `ByteOrderRead`
/// for free.
impl<R: Read + ?Sized> ByteOrderRead for R {}

macro_rules! write_num_bytes {
    ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => {{
        assert!($size <= $dst.len());
        unsafe {
            // N.B. https://github.com/rust-lang/rust/issues/22776
            let bytes = *(&$n.$which() as *const _ as *const [u8; $size]);
            ptr::copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    }};
}

/// Extends `std::io::Write` with methods for writing numbers in `BigEndian`.
pub trait ByteOrderWrite: Write {
    /// Writes an unsigned 8 bit integer to the underlying writer.
    #[inline]
    fn write_u8(&mut self, n: u8) -> Result<()> {
        self.write_all(&[n])
    }

    /// Writes an unsigned 16 bit integer to the underlying writer.
    #[inline]
    fn write_u16(&mut self, n: u16) -> Result<()> {
        let mut buf = [0; 2];
        write_num_bytes!(u16, 2, n, &mut buf, to_be);
        self.write_all(&buf)
    }

    /// Writes an unsigned 32 bit integer to the underlying writer.
    #[inline]
    fn write_u32(&mut self, n: u32) -> Result<()> {
        let mut buf = [0; 4];
        write_num_bytes!(u32, 4, n, &mut buf, to_be);
        self.write_all(&buf)
    }

    /// Writes an unsigned 64 bit integer to the underlying writer.
    #[inline]
    fn write_u64(&mut self, n: u64) -> Result<()> {
        let mut buf = [0; 8];
        write_num_bytes!(u64, 8, n, &mut buf, to_be);
        self.write_all(&buf)
    }

    /// Writes a signed 8 bit integer to the underlying writer.
    #[inline]
    fn write_i8(&mut self, n: i8) -> Result<()> {
        self.write_all(&[n as u8])
    }

    /// Writes a signed 16 bit integer to the underlying writer.
    #[inline]
    fn write_i16(&mut self, n: i16) -> Result<()> {
        self.write_u16(n as u16)
    }

    /// Writes a signed 32 bit integer to the underlying writer.
    #[inline]
    fn write_i32(&mut self, n: i32) -> Result<()> {
        self.write_u32(n as u32)
    }

    /// Writes a signed 64 bit integer to the underlying writer.
    #[inline]
    fn write_i64(&mut self, n: i64) -> Result<()> {
        self.write_u64(n as u64)
    }

    /// Writes a IEEE754 single-precision (4 bytes) floating point number.
    #[inline]
    fn write_f32(&mut self, n: f32) -> Result<()> {
        let n = unsafe { *(&n as *const f32 as *const u32) };
        self.write_u32(n)
    }

    /// Writes a IEEE754 double-precision (8 bytes) floating point number.
    #[inline]
    fn write_f64(&mut self, n: f64) -> Result<()> {
        let n = unsafe { *(&n as *const f64 as *const u64) };
        self.write_u64(n)
    }

    /// Writes a copyable value in big endian device.
    #[cfg(target_endian = "big")]
    #[inline]
    fn write<T: Copy>(&mut self, v: T) -> Result<()> {
        unsafe {
            let buf = slice::from_raw_parts(&v as *const T as *const u8, mem::size_of::<T>());
            self.write_all(&buf)
        }
    }

    /// Writes a copyable value in little endian device.
    #[cfg(target_endian = "little")]
    #[inline]
    fn write<T: Copy>(&mut self, mut v: T) -> Result<()> {
        unsafe {
            let buf = slice::from_raw_parts_mut(&mut v as *mut T as *mut u8, mem::size_of::<T>());
            buf.reverse();
            self.write_all(&buf)
        }
    }
}

/// All types that implement `Write` get methods defined in `ByteOrderWrite`
/// for free.
impl<W: Write + ?Sized> ByteOrderWrite for W {}
