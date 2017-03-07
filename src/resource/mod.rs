pub mod errors;
pub mod archive;
pub mod cache;
pub mod texture;

pub use self::archive::{Read, Seek, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::texture::Texture;

use utility::Handle;

impl_handle!(ResourceHandle);

pub trait Resource {
    fn from_bytes(bytes: &[u8]) -> self::errors::Result<Self> where Self: Sized;
}