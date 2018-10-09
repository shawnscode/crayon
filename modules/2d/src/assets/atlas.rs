use std::sync::Arc;

use uuid::Uuid;

use crayon::math::prelude::*;
use crayon::res::prelude::*;
use crayon::utils::{FastHashMap, HashValue};

use super::texture::TextureHandle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtlasVertex {
    pub position: Vector2<f32>,
    pub uv: Vector2<f32>,
}

impl_handle!(AtlasHandle);
impl_handle!(AtlasSpriteHandle);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableAtlas {
    /// The universal-uniqued id of underlying texture.
    pub texture: Uuid,
    /// The sprites of this atlas.
    pub sprites: Vec<SerializableSprite>,
    /// The array containing sprite mesh vertex positions and texcoords.
    pub vertices: Vec<AtlasVertex>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableSprite {
    /// The name
    pub name: String,
    /// The inclusive start index of sprite vertices
    pub start: usize,
    /// The exclusive end index of sprite vertices.
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Atlas {
    pub texture: TextureHandle,
    pub sprites: FastHashMap<HashValue<str>, Arc<Vec<AtlasVertex>>>,
}

pub struct AtlasSystemShared {
    atlas: Registry<AtlasHandle, AtlasLoader>,
}
