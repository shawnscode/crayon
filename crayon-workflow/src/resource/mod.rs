pub mod bytes;
pub mod texture;

#[derive(Debug)]
pub enum Resource {
    Binary,
    Texture,
}