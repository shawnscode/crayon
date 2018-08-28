use crayon::math;
use crayon::video::assets::texture::TextureHandle;

#[derive(Debug, Copy, Clone)]
pub struct SimpleMaterial {
    pub diffuse: math::Color<f32>,
    pub diffuse_texture: Option<TextureHandle>,
    pub ambient: math::Color<f32>,
    pub specular: math::Color<f32>,
    pub shininess: f32,
}

impl Default for SimpleMaterial {
    fn default() -> Self {
        SimpleMaterial {
            diffuse_texture: None,
            ambient: math::Color::white(),
            diffuse: math::Color::white(),
            specular: math::Color::black(),
            shininess: 0.0,
        }
    }
}
