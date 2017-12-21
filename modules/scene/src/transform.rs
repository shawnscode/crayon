use crayon::{ecs, math};
use crayon::math::Transform as _Transform;

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
}

/// Declare `Transform` as component with compact vec storage.
impl ecs::Component for Transform {
    type Arena = ecs::VecArena<Transform>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform { decomposed: math::Decomposed::one() }
    }
}

impl Transform {
    #[inline(always)]
    pub fn decomposed(&self) -> math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>> {
        self.decomposed
    }

    #[inline(always)]
    pub fn scale(&self) -> f32 {
        self.decomposed.scale
    }

    #[inline(always)]
    pub fn set_scale(&mut self, scale: f32) {
        self.decomposed.scale = scale;
    }

    #[inline(always)]
    pub fn position(&self) -> math::Vector3<f32> {
        self.decomposed.disp
    }

    #[inline(always)]
    pub fn set_position<T>(&mut self, position: T)
        where T: Into<math::Vector3<f32>>
    {
        self.decomposed.disp = position.into();
    }

    #[inline(always)]
    pub fn translate<T>(&mut self, disp: T)
        where T: Into<math::Vector3<f32>>
    {
        self.decomposed.disp += disp.into();
    }

    #[inline(always)]
    pub fn rotation(&self) -> math::Quaternion<f32> {
        self.decomposed.rot
    }

    #[inline(always)]
    pub fn set_rotation<T>(&mut self, rotation: T)
        where T: Into<math::Quaternion<f32>>
    {
        self.decomposed.rot = rotation.into();
    }

    #[inline(always)]
    pub fn rotate<T>(&mut self, rotate: T)
        where T: Into<math::Quaternion<f32>>
    {
        self.decomposed.rot = rotate.into() * self.decomposed.rot;
    }
}