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
        match *self {
            AssetState::Ready(_) => true,
            _ => false,
        }
    }
}

pub(crate) type AssetMeshState = AssetState<self::mesh::MeshParams>;
pub(crate) type AssetTextureState = AssetState<self::texture::TextureParams>;
pub(crate) type AssetRenderTextureState = AssetState<self::texture::RenderTextureStateObject>;

pub mod prelude {
    pub use super::surface::{SurfaceHandle, SurfaceScissor, SurfaceSetup, SurfaceViewport};

    pub use super::shader::{Attribute, AttributeLayout, AttributeLayoutBuilder, BlendFactor,
                            BlendValue, Comparison, CullFace, Equation, FrontFaceOrder,
                            RenderState, ShaderHandle, ShaderParams, ShaderSetup, UniformVariable,
                            UniformVariableType};

    pub use super::texture::{RenderTextureFormat, RenderTextureHandle, RenderTextureSetup,
                             TextureAddress, TextureFilter, TextureFormat, TextureHandle,
                             TextureHint, TextureParams, TextureSetup};

    pub use super::mesh::{IndexFormat, MeshHandle, MeshHint, MeshIndex, MeshParams, MeshPrimitive,
                          MeshSetup, VertexFormat, VertexLayout};

    pub(crate) use super::{AssetMeshState, AssetRenderTextureState, AssetState, AssetTextureState};
}
