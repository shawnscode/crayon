//! A collection of serialization and deserialization functions
//! that use the `serde` crate for the serializable and deserializable
//! implementation.

pub mod encode;
pub mod decode;
pub mod error;

pub use self::decode::SizeLimit;

use std::io::{Write, Read};
use serde;
use byteorder;

/// Serializes an object directly into a `Writer`.
///
/// If this returns an `Error`, assume that the writer is in an invalid state,
/// as writing could bail out in the middle of serializing.
pub fn serialize_into<W, T, E>(writer: &mut W, value: &T) -> error::Result<()>
    where W: Write,
          T: serde::Serialize,
          E: byteorder::ByteOrder
{
    let mut serializer = encode::Serializer::<_, E>::new(writer);
    serde::Serialize::serialize(value, &mut serializer)
}

/// Serializes a serializable object into a `Vec` of bytes.
pub fn serialize<T>(value: &T) -> error::Result<Vec<u8>>
    where T: serde::Serialize
{
    let mut writer = Vec::new();
    serialize_into::<_, _, byteorder::BigEndian>(&mut writer, value)?;
    Ok(writer)
}

/// Deserializes an object directly from a `Buffer`ed Reader.
///
/// If the provided `SizeLimit` is reached, the deserialization will bail immediately.
/// A SizeLimit can help prevent an attacker from flooding your server with
/// a neverending stream of values that runs your server out of memory.
///
/// If this returns an `Error`, assume that the buffer that you passed
/// in is in an invalid state, as the error could be returned during any point
/// in the reading.
pub fn deserialize_from<R, T, E: byteorder::ByteOrder>(reader: &mut R,
                                                       size_limit: SizeLimit)
                                                       -> error::Result<T>
    where R: Read,
          T: serde::Deserialize
{
    let mut deserializer = decode::Deserializer::<_, E>::new(reader, size_limit);
    serde::Deserialize::deserialize(&mut deserializer)
}

/// Deserializes a slice of bytes into an object.
///
/// This method does not have a size-limit because if you already have the bytes
/// in memory, then you don't gain anything by having a limiter.
pub fn deserialize<T>(bytes: &[u8]) -> error::Result<T>
    where T: serde::Deserialize
{
    let mut reader = bytes;
    deserialize_from::<_, _, byteorder::BigEndian>(&mut reader, SizeLimit::Infinite)
}


#[cfg(test)]
mod test {
    use super::*;
    use utility::memory::variant;

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    enum Enum {
        Placeholder,
        Depth(u32),
        Position { x: i32, y: i32 },
    }

    #[test]
    fn enumerations() {
        let a = Enum::Placeholder;
        let b = Enum::Depth(24);
        let c = Enum::Position { x: 1, y: 2 };

        let a_bytes = serialize(&a).unwrap();
        assert_eq!(a_bytes.len(), 1);
        assert_eq!(&a_bytes, &[0]);
        deserialize::<Enum>(&a_bytes).unwrap();
        assert_eq!(a, deserialize::<Enum>(&a_bytes).unwrap());

        let b_bytes = serialize(&b).unwrap();
        assert_eq!(b_bytes.len(), 5);
        assert_eq!(&b_bytes, &[1, 0, 0, 0, 24]);
        assert_eq!(b, deserialize::<Enum>(&b_bytes).unwrap());

        let c_bytes = serialize(&c).unwrap();
        assert_eq!(c_bytes.len(), 9);
        assert_eq!(&c_bytes, &[2, 0, 0, 0, 1, 0, 0, 0, 2]);
        assert_eq!(c, deserialize::<Enum>(&c_bytes).unwrap());
    }

    #[test]
    fn variant() {
        let vs1 = variant::VariantStr::from("u_Position");
        let vs1_bytes = serialize(&vs1).unwrap();
        assert_eq!(vs1_bytes.len(), 16);
        assert_eq!(deserialize::<variant::VariantStr>(&vs1_bytes).unwrap().as_str(),
                   "u_Position");
        assert_eq!(deserialize::<variant::VariantStr>(&vs1_bytes).unwrap().as_str(),
                   "u_Position");

        let vs2 = variant::VariantStr::from("u_Texture_Diffuse");
        let vs2_bytes = serialize(&vs2).unwrap();
        assert_eq!(vs2_bytes.len(), 32);
        assert_eq!(deserialize::<variant::VariantStr>(&vs2_bytes).unwrap().as_str(),
                   "u_Texture_Diffuse");
        assert_eq!(deserialize::<variant::VariantStr>(&vs2_bytes).unwrap().as_str(),
                   "u_Texture_Diffuse");

        let vs3 = variant::VariantStr::from("u_MainTexture");
        let vs3_bytes = serialize(&vs3).unwrap();
        assert_eq!(vs3_bytes.len(), 16);
        assert_eq!(deserialize::<variant::VariantStr>(&vs3_bytes).unwrap().as_str(),
                   "u_MainTexture");
        assert_eq!(deserialize::<variant::VariantStr>(&vs3_bytes).unwrap().as_str(),
                   "u_MainTexture");
    }
}
