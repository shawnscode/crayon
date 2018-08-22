use crayon::math;
use crayon::video::assets::texture::TextureHandle;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub texture: TextureHandle,
    pub ambient: math::Color<f32>,
    pub diffuse: math::Color<f32>,
    pub specular: math::Color<f32>,
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            texture: TextureHandle::default(),
            ambient: math::Color::white(),
            diffuse: math::Color::white(),
            specular: math::Color::black(),
            shininess: 0.0,
        }
    }
}
