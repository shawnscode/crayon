use std::io::{Write, Read};
use std::path::Path;
use std::fs;

use toml;
use serde;

use errors::*;

use crayon::math;

pub fn load<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
    let mut next = value;
    for i in path {
        if let Some(leaf) = next.get(i) {
            next = leaf;
        } else {
            return None;
        }
    }
    Some(next)
}

#[inline]
pub fn load_as_i32(value: &toml::Value, path: &[&str]) -> Option<i32> {
    load(&value, &path)
        .and_then(|v| v.as_integer())
        .map(|v| v as i32)
}

#[inline]
pub fn load_as_u32(value: &toml::Value, path: &[&str]) -> Option<u32> {
    load(&value, &path)
        .and_then(|v| v.as_integer())
        .map(|v| v as u32)
}

#[inline]
pub fn load_as_f32(value: &toml::Value, path: &[&str]) -> Option<f32> {
    load(&value, &path)
        .and_then(|v| v.as_float())
        .map(|v| v as f32)
}

#[inline]
pub fn load_as_str<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a str> {
    load(&value, &path).and_then(|v| v.as_str())
}

#[inline]
pub fn load_as_array<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a Vec<toml::Value>> {
    load(&value, &path).and_then(|v| v.as_array())
}

#[inline]
pub fn load_as_vec2(value: &toml::Value, path: &[&str]) -> Option<math::Vector2<f32>> {
    load_as_array(&value, &path).and_then(|v| parse_array_as_vec2(v))
}

#[inline]
pub fn load_as_vec3(value: &toml::Value, path: &[&str]) -> Option<math::Vector3<f32>> {
    load_as_array(&value, &path).and_then(|v| parse_array_as_vec3(v))
}

#[inline]
pub fn load_as_vec4(value: &toml::Value, path: &[&str]) -> Option<math::Vector4<f32>> {
    load_as_array(&value, &path).and_then(|v| parse_array_as_vec4(v))
}

pub fn load_as_mat2(value: &toml::Value, path: &[&str]) -> Option<math::Matrix2<f32>> {
    if let Some(v) = load_as_array(&value, &path) {
        let cols: Vec<_> = v.iter()
            .filter_map(|v| v.as_array())
            .filter_map(|v| parse_array_as_vec2(&v))
            .collect();

        if cols.len() >= 2 {
            return Some(math::Matrix2::from_cols(cols[0], cols[1]));
        }
    }

    None
}

pub fn load_as_mat3(value: &toml::Value, path: &[&str]) -> Option<math::Matrix3<f32>> {
    if let Some(v) = load_as_array(&value, &path) {
        let cols: Vec<_> = v.iter()
            .filter_map(|v| v.as_array())
            .filter_map(|v| parse_array_as_vec3(&v))
            .collect();

        if cols.len() >= 3 {
            return Some(math::Matrix3::from_cols(cols[0], cols[1], cols[2]));
        }
    }

    None
}

pub fn load_as_mat4(value: &toml::Value, path: &[&str]) -> Option<math::Matrix4<f32>> {
    if let Some(v) = load_as_array(&value, &path) {
        let cols: Vec<_> = v.iter()
            .filter_map(|v| v.as_array())
            .filter_map(|v| parse_array_as_vec4(&v))
            .collect();

        if cols.len() >= 4 {
            return Some(math::Matrix4::from_cols(cols[0], cols[1], cols[2], cols[3]));
        }
    }

    None
}

pub fn serialize<T, P>(value: &T, path: P) -> Result<()>
    where T: serde::Serialize,
          P: AsRef<Path>
{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())?;

    let serialization = toml::to_string(&value)?;
    file.write(serialization.as_ref())?;
    file.flush()?;

    Ok(())
}

pub fn deserialize<T, P>(path: P) -> Result<T>
    where T: serde::de::DeserializeOwned,
          P: AsRef<Path>
{
    let path = path.as_ref();
    if !path.exists() {
        bail!(ErrorKind::FileNotFound);
    }

    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut serialization = String::new();
    file.read_to_string(&mut serialization)?;

    Ok(toml::from_str(&serialization)?)
}

fn parse_array_as_vec2<'a>(value: &'a Vec<toml::Value>) -> Option<math::Vector2<f32>> {
    let vec = parse_array_f32(&value);
    if vec.len() >= 2 {
        Some(math::Vector2::new(vec[0], vec[1]))
    } else {
        None
    }
}

fn parse_array_as_vec3<'a>(value: &'a Vec<toml::Value>) -> Option<math::Vector3<f32>> {
    let vec = parse_array_f32(&value);
    if vec.len() >= 3 {
        Some(math::Vector3::new(vec[0], vec[1], vec[2]))
    } else {
        None
    }
}

fn parse_array_as_vec4<'a>(value: &'a Vec<toml::Value>) -> Option<math::Vector4<f32>> {
    let vec = parse_array_f32(&value);
    if vec.len() >= 4 {
        Some(math::Vector4::new(vec[0], vec[1], vec[2], vec[3]))
    } else {
        None
    }
}

fn parse_array_f32<'a>(value: &'a Vec<toml::Value>) -> Vec<f32> {
    value
        .iter()
        .filter_map(|v| v.as_float())
        .map(|v| v as f32)
        .collect()
}