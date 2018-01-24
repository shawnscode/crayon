use utils::Color;

#[derive(Debug, Clone, Copy)]
pub struct Light {
    /// Is this light enable.
    pub enable: bool,
    /// Color of the light.
    pub color: Color,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Light source
    pub source: LightSource,
}

/// Enumeration for all light sources.
#[derive(Debug, Clone, Copy)]
pub enum LightSource {
    /// A direcitonal light.
    Directional,
    /// A point light.
    Point {
        /// Maximum raidus of the point light's affected data.
        radius: f32,
        /// Smoothness of the light-to-dark transition from the center to the radius.
        smoothness: f32,
    },
}

impl Default for Light {
    fn default() -> Self {
        Light {
            enable: true,
            color: Color::white(),
            intensity: 1.0,
            source: LightSource::Directional,
        }
    }
}
