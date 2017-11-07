//! Build-in assets including `Texture`, `Atlas`, `Shader` etc..

pub mod bytes;
pub mod texture;
pub mod texture_sys;

pub use self::bytes::Bytes;

pub use self::texture::Texture;
pub use self::texture_sys::TextureSystem;

// pub fn register(resource: &mut resource::ResourceSystem,
//                 video: Arc<graphics::GraphicsSystemShared>) {
//     resource.register::<Bytes>(0);
//     resource.register::<Texture>(0);
//     resource.register_extern_system(GraphicsResourceSystem::<graphics::TextureHandle>::new(video));
// }