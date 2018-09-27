use crayon::math::prelude::Color;
use crayon::video::assets::texture::TextureHandle;

#[derive(Debug, Copy, Clone)]
pub struct SimpleMaterial {
    pub ambient: Color<f32>,
    pub diffuse: Color<f32>,
    pub diffuse_texture: Option<TextureHandle>,
    pub specular: Color<f32>,
    pub specular_texture: Option<TextureHandle>,
    pub shininess: f32,
}

impl Default for SimpleMaterial {
    fn default() -> Self {
        SimpleMaterial {
            ambient: Color::white(),
            diffuse: Color::white(),
            diffuse_texture: None,
            specular: Color::black(),
            specular_texture: None,
            shininess: 0.0,
        }
    }
}
