use crayon::math;
use crayon::video::assets::texture::TextureHandle;

#[derive(Debug, Copy, Clone)]
pub struct SimpleMaterial {
    pub ambient: math::Color<f32>,
    pub diffuse: math::Color<f32>,
    pub diffuse_texture: Option<TextureHandle>,
    pub specular: math::Color<f32>,
    pub specular_texture: Option<TextureHandle>,
    pub shininess: f32,
}

impl Default for SimpleMaterial {
    fn default() -> Self {
        SimpleMaterial {
            ambient: math::Color::white(),
            diffuse: math::Color::white(),
            diffuse_texture: None,
            specular: math::Color::black(),
            specular_texture: None,
            shininess: 0.0,
        }
    }
}
