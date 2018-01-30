pub mod surface;
pub mod shader;
pub mod texture;
pub mod texture_loader;
#[macro_use]
pub mod mesh;
pub mod mesh_loader;

use std::sync::Arc;

pub(crate) enum AssetState<T>
where
    T: Sized + 'static,
{
    NotReady,
    Ready(Arc<T>),
    Err(String),
}

impl<T> AssetState<T>
where
    T: Sized,
{
    pub fn ready(v: T) -> Self {
        AssetState::Ready(Arc::new(v))
    }

    pub fn is_ready(&self) -> bool {
        match self {
            &AssetState::Ready(_) => true,
            _ => false,
        }
    }

    pub fn clone(&self) -> Option<Arc<T>> {
        match self {
            &AssetState::Ready(ref v) => Some(v.clone()),
            _ => None,
        }
    }
}

pub(crate) type AssetShaderState = AssetState<self::shader::ShaderStateObject>;
pub(crate) type AssetMeshState = AssetState<self::mesh::MeshStateObject>;
pub(crate) type AssetTextureState = AssetState<self::texture::TextureStateObject>;
pub(crate) type AssetRenderTextureState = AssetState<self::texture::RenderTextureStateObject>;
