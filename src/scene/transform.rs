use ecs;
use math;
use math::Transform as _Transform;
use math::{EuclideanSpace, Matrix, One, Rotation};

use scene::node::Node;
use scene::errors::*;

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object. We use a left handed, y-up world coordinate system.
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
        Transform {
            decomposed: math::Decomposed::one(),
        }
    }
}

impl Transform {
    /// Get the scale component in local space.
    #[inline]
    pub fn scale(&self) -> f32 {
        self.decomposed.scale
    }

    /// Set the scale component in local space.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        self.decomposed.scale = scale;
    }

    #[inline]
    pub fn position(&self) -> math::Vector3<f32> {
        self.decomposed.disp
    }

    #[inline]
    pub fn set_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        self.decomposed.disp = position.into();
    }

    #[inline]
    pub fn translate<T>(&mut self, disp: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        self.decomposed.disp += disp.into();
    }

    #[inline]
    pub fn rotation(&self) -> math::Quaternion<f32> {
        self.decomposed.rot
    }

    #[inline]
    pub fn set_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        self.decomposed.rot = rotation.into();
    }

    #[inline]
    pub fn rotate<T>(&mut self, rotate: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        self.decomposed.rot = rotate.into() * self.decomposed.rot;
    }
}

impl Transform {
    /// Get the transform matrix from local space to world space.
    pub fn world_matrix<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        let decomposed = Transform::world_decomposed(tree, arena, handle)?;
        Ok(math::Matrix4::from(decomposed))
    }

    /// Get the view matrix from world space to view space.
    pub fn world_view_matrix<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        let decomposed = Transform::world_decomposed(tree, arena, handle)?;
        let it = math::Matrix4::from_translation(-decomposed.disp);
        let ir = math::Matrix4::from(decomposed.rot).transpose();
        // M = ( T * R ) ^ -1
        Ok(ir * it)
    }

    /// Get the transform matrix from world space to local space.
    pub fn inverse_world_matrix<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        let decomposed = Transform::world_decomposed(tree, arena, handle)?;
        if let Some(inverse) = decomposed.inverse_transform() {
            Ok(math::Matrix4::from(inverse))
        } else {
            bail!(ErrorKind::CanNotInverseTransform);
        }
    }

    /// Set position of `Transform` in world space.
    pub fn set_world_position<T1, T2, T3>(
        tree: &T1,
        arena: &mut T2,
        handle: ecs::Entity,
        disp: T3,
    ) -> Result<()>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        if arena.get(handle).is_none() {
            bail!(ErrorKind::NonTransformFound);
        }

        let disp = disp.into();
        unsafe {
            if tree.get(handle).is_none() {
                arena.get_unchecked_mut(handle).set_position(disp);
            } else {
                let mut ancestors_disp = math::Vector3::new(0.0, 0.0, 0.0);
                for v in Node::ancestors(tree, handle) {
                    if let Some(transform) = arena.get(v) {
                        ancestors_disp += transform.position();
                    }
                }

                arena
                    .get_unchecked_mut(handle)
                    .set_position(disp - ancestors_disp);
            }

            Ok(())
        }
    }

    /// Get position of `Transform` in world space.
    pub fn world_position<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        if let Some(transform) = arena.get(handle) {
            let mut disp = transform.position();
            for v in Node::ancestors(tree, handle) {
                if let Some(ancestor) = arena.get(v) {
                    disp += ancestor.position();
                }
            }
            Ok(disp)
        } else {
            bail!(ErrorKind::NonTransformFound);
        }
    }

    /// Set uniform scale of `Transform` in world space.
    pub fn set_world_scale<T1, T2>(
        tree: &T1,
        arena: &mut T2,
        handle: ecs::Entity,
        scale: f32,
    ) -> Result<()>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
    {
        if arena.get(handle).is_none() {
            bail!(ErrorKind::NonTransformFound);
        }

        unsafe {
            if tree.get(handle).is_none() {
                arena.get_unchecked_mut(handle).set_scale(scale);
            } else {
                let mut ancestors_scale = 1.0;
                for v in Node::ancestors(tree, handle) {
                    if let Some(transform) = arena.get(v) {
                        ancestors_scale *= transform.scale();
                    }
                }

                if ancestors_scale < ::std::f32::EPSILON {
                    arena.get_unchecked_mut(handle).set_scale(scale);
                } else {
                    arena
                        .get_unchecked_mut(handle)
                        .set_scale(scale / ancestors_scale);
                }
            }

            Ok(())
        }
    }

    /// Get the scale of `Transform` in world space.
    pub fn world_scale<T1, T2>(tree: &T1, arena: &mut T2, handle: ecs::Entity) -> Result<f32>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
    {
        if let Some(transform) = arena.get(handle) {
            let mut scale = transform.scale();
            for v in Node::ancestors(tree, handle) {
                if let Some(ancestor) = arena.get(v) {
                    scale *= ancestor.scale();
                }
            }
            Ok(scale)
        } else {
            bail!(ErrorKind::NonTransformFound);
        }
    }

    /// Set rotation of `Transform` in world space.
    pub fn set_world_rotation<T1, T2, T3>(
        tree: &T1,
        arena: &mut T2,
        handle: ecs::Entity,
        rotation: T3,
    ) -> Result<()>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
        T3: Into<math::Quaternion<f32>>,
    {
        if arena.get(handle).is_none() {
            bail!(ErrorKind::NonTransformFound);
        }

        unsafe {
            if tree.get(handle).is_none() {
                arena.get_unchecked_mut(handle).set_rotation(rotation);
            } else {
                let mut ancestors_rotation = math::Quaternion::one();
                for v in Node::ancestors(tree, handle) {
                    if let Some(transform) = arena.get(v) {
                        ancestors_rotation = ancestors_rotation * transform.rotation();
                    }
                }

                arena
                    .get_unchecked_mut(handle)
                    .set_rotation(rotation.into() * ancestors_rotation.invert());
            }

            Ok(())
        }
    }

    /// Get rotation of `Transform` in world space.
    pub fn world_rotation<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Quaternion<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        if let Some(transform) = arena.get(handle) {
            let mut rotation = transform.rotation();
            for v in Node::ancestors(tree, handle) {
                if let Some(ancestor) = arena.get(v) {
                    rotation = rotation * ancestor.rotation();
                }
            }
            Ok(rotation)
        } else {
            bail!(ErrorKind::NonTransformFound);
        }
    }

    /// Rotate the transform so the forward vector points at target's current position.
    pub fn look_at<T1, T2, T3, T4>(
        tree: &T1,
        arena: &mut T2,
        handle: ecs::Entity,
        dst: T3,
        up: T4,
    ) -> Result<()>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
        T3: Into<math::Vector3<f32>>,
        T4: Into<math::Vector3<f32>>,
    {
        let eye = math::Point3::from_vec(Transform::world_position(tree, arena, handle)?);
        let center = math::Point3::from_vec(dst.into());
        let rotation = math::Quaternion::look_at(center - eye, -up.into());
        Transform::set_world_rotation(tree, arena, handle, rotation)
    }

    #[allow(dead_code)]
    fn set_world_decomposed<T1, T2>(
        tree: &T1,
        arena: &mut T2,
        handle: ecs::Entity,
        decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
    ) -> Result<()>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::ArenaMut<Transform>,
    {
        let relative = Transform::world_decomposed(tree, arena, handle)?;

        if let Some(inverse) = relative.inverse_transform() {
            unsafe {
                arena.get_unchecked_mut(handle).decomposed = inverse.concat(&decomposed);
            }
            Ok(())
        } else {
            bail!(ErrorKind::CanNotInverseTransform);
        }
    }

    fn world_decomposed<T1, T2>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
    ) -> Result<math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        if let Some(transform) = arena.get(handle) {
            let mut decomposed = transform.decomposed;
            for v in Node::ancestors(tree, handle) {
                if let Some(ancestor) = arena.get(v) {
                    decomposed = ancestor.decomposed.concat(&decomposed);
                }
            }
            Ok(decomposed)
        } else {
            bail!(ErrorKind::NonTransformFound);
        }
    }
}

impl Transform {
    /// Transforms position from local space to world space.
    pub fn transform_point<T1, T2, T3>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let decomposed = Transform::world_decomposed(tree, arena, handle)?;
        // M = T * R * S
        Ok(decomposed.rot * (v.into() * decomposed.scale) + decomposed.disp)
    }

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    pub fn transform_vector<T1, T2, T3>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let decomposed = Transform::world_decomposed(tree, arena, handle)?;
        Ok(decomposed.transform_vector(v.into()))
    }

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    pub fn transform_direction<T1, T2, T3>(
        tree: &T1,
        arena: &T2,
        handle: ecs::Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let rotation = Transform::world_rotation(tree, arena, handle)?;
        Ok(rotation * v.into())
    }

    /// Return the up direction in world space, which is looking down the positive y-axis.
    pub fn up<T1, T2>(tree: &T1, arena: &T2, handle: ecs::Entity) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        Transform::transform_direction(tree, arena, handle, math::Vector3::new(0.0, 1.0, 0.0))
    }

    /// Return the forward direction in world space, which is looking down the positive z-axis.
    pub fn forward<T1, T2>(tree: &T1, arena: &T2, handle: ecs::Entity) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        Transform::transform_direction(tree, arena, handle, math::Vector3::new(0.0, 0.0, 1.0))
    }

    /// Return the right direction in world space, which is looking down the positive x-axis.
    pub fn right<T1, T2>(tree: &T1, arena: &T2, handle: ecs::Entity) -> Result<math::Vector3<f32>>
    where
        T1: ecs::Arena<Node>,
        T2: ecs::Arena<Transform>,
    {
        Transform::transform_direction(tree, arena, handle, math::Vector3::new(1.0, 0.0, 0.0))
    }
}
