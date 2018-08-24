pub mod shader;
pub mod surface;
pub mod texture;
pub mod texture_loader;
#[macro_use]
pub mod mesh;
pub mod mesh_loader;

pub mod prelude {
    pub use super::surface::{SurfaceHandle, SurfaceParams, SurfaceScissor, SurfaceViewport};

    pub use super::shader::{
        Attribute, AttributeLayout, AttributeLayoutBuilder, BlendFactor, BlendValue, Comparison,
        CullFace, Equation, FrontFaceOrder, RenderState, ShaderHandle, ShaderParams,
        UniformVariable, UniformVariableLayout, UniformVariableLayoutBuilder, UniformVariableType,
    };

    pub use super::texture::{
        RenderTextureFormat, RenderTextureHandle, RenderTextureParams, TextureData, TextureFilter,
        TextureFormat, TextureHandle, TextureHint, TextureParams, TextureWrap,
    };

    pub use super::mesh::{
        IndexFormat, MeshData, MeshHandle, MeshHint, MeshIndex, MeshParams, MeshPrimitive,
        VertexFormat, VertexLayout,
    };
}
