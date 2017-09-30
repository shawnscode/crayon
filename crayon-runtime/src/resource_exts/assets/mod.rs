//! Build-in assets including `Texture`, `Atlas`, `Shader` etc..

pub mod bytes;
pub mod texture;
pub mod atlas;
pub mod mesh;
pub mod shader;
pub mod material;

use super::Ptr;

pub use self::bytes::Bytes;
pub type BytesPtr = Ptr<Bytes>;
// declare_resource!(Bytes);

pub use self::texture::Texture;
pub type TexturePtr = Ptr<Texture>;
// declare_resource!(Texture);

pub use self::atlas::Atlas;
pub type AtlasPtr = Ptr<Atlas>;
// declare_resource!(Atlas);

pub use self::shader::Shader;
pub type ShaderPtr = Ptr<Shader>;
// declare_resource!(Shader);

pub use self::material::Material;
pub type MaterialPtr = Ptr<Material>;
// declare_resource!(Material);

pub use self::mesh::Mesh;
pub type MeshPtr = Ptr<Mesh>;
// declare_resource!(Mesh);