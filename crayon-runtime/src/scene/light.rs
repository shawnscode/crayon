/// The `Light` component that support `Point` and `Directional` right now.

use math;
use ecs::HashMapArena;

/// Enumeration for all light components.
#[derive(Debug, Clone, Copy)]
pub enum Light {
    /// A direcitonal light.
    Directional(DirectionalLight),
    /// A point light.
    Point(PointLight),
}

/// Declare `Light` as component.
declare_component!(Light, HashMapArena);

impl Light {
    pub fn is_enable(&self) -> bool {
        match self {
            &Light::Directional(ref v) => v.enable,
            &Light::Point(ref v) => v.enable,
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        match self {
            &mut Light::Directional(ref mut v) => v.enable = enable,
            &mut Light::Point(ref mut v) => v.enable = enable,
        }
    }
}

/// A directional light source.
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    /// Is this light enable.
    pub enable: bool,
    /// Color of the light.
    pub color: math::Color,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            enable: true,
            color: math::Color::white(),
            intensity: 1.0,
        }
    }
}

impl From<DirectionalLight> for Light {
    fn from(src: DirectionalLight) -> Self {
        Light::Directional(src)
    }
}

/// A point light source.
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    /// Is this light enable.
    pub enable: bool,
    /// Color of the light.
    pub color: math::Color,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Maximum raidus of the point light's affected data.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the radius.
    pub smoothness: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            enable: true,
            color: math::Color::white(),
            intensity: 1.0,
            radius: 1.0,
            smoothness: 1.0,
        }
    }
}

impl From<PointLight> for Light {
    fn from(src: PointLight) -> Light {
        Light::Point(src)
    }
}