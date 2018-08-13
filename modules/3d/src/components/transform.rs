use std::collections::HashMap;

use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::Transform as _Transform;
use crayon::math::{Matrix, One, Rotation};

use components::node::Node;
use errors::*;

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object. We use a left handed, y-up world coordinate system.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
}

/// Declare `Transform` as component with compact vec storage.
impl Component for Transform {
    type Arena = VecArena<Transform>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            decomposed: math::Decomposed::one(),
        }
    }
}

impl Transform {
    #[inline]
    pub fn matrix(&self) -> math::Matrix4<f32> {
        math::Matrix4::from(self.decomposed)
    }

    /// Gets the scale component in local space.
    #[inline]
    pub fn scale(&self) -> f32 {
        self.decomposed.scale
    }

    /// Sets the scale component in local space.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        self.decomposed.scale = scale;
    }

    /// Gets the position component in local space.
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

    #[inline]
    pub fn transform_direction<T>(&self, v: T) -> math::Vector3<f32>
    where
        T: Into<math::Vector3<f32>>,
    {
        self.decomposed.rot * v.into()
    }

    pub fn up(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(0.0, 1.0, 0.0))
    }

    pub fn forward(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(0.0, 0.0, 1.0))
    }

    pub fn right(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(1.0, 0.0, 0.0))
    }
}

impl Transform {
    pub fn world_transforms<T1, T2>(
        entities: Entities,
        nodes: &T1,
        transforms: &T2,
    ) -> HashMap<Entity, Transform>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let mut result = HashMap::new();

        unsafe {
            for v in entities.with_2::<Node, Transform>() {
                if nodes.get_unchecked(v).is_root() {
                    let d = transforms.get_unchecked(v).decomposed;
                    result.insert(v, Transform { decomposed: d });
                    Self::world_transforms_from(nodes, transforms, v, d, &mut result);
                }
            }
        }

        result
    }

    unsafe fn world_transforms_from<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        ancestor: Entity,
        decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
        result: &mut HashMap<Entity, Transform>,
    ) where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        for child in Node::children(nodes, ancestor) {
            let d = decomposed.concat(&transforms.get_unchecked(child).decomposed);
            result.insert(child, Transform { decomposed: d });
            Self::world_transforms_from(nodes, transforms, child, d, result);
        }
    }
}

impl Transform {
    /// Get the transform matrix from local space to world space.
    pub fn world_matrix<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        Ok(math::Matrix4::from(decomposed))
    }

    /// Get the transform matrix from world space to local space.
    pub fn inverse_world_matrix<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        if let Some(inverse) = decomposed.inverse_transform() {
            Ok(math::Matrix4::from(inverse))
        } else {
            Err(Error::CanNotInverseTransform)
        }
    }

    /// Get the view matrix from world space to view space.
    pub fn world_view_matrix<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        let it = math::Matrix4::from_translation(-decomposed.disp);
        let ir = math::Matrix4::from(decomposed.rot).transpose();
        // M = ( T * R ) ^ -1
        Ok(ir * it)
    }

    /// Gets the inverse view matrix which transform vector from view space to world space.
    pub fn inverse_world_view_matrix<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Matrix4<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        let t = math::Matrix4::from_translation(decomposed.disp);
        let r = math::Matrix4::from(decomposed.rot);
        Ok(t * r)
    }

    /// Set position of `Transform` in world space.
    pub fn set_world_position<T1, T2, T3>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        disp: T3,
    ) -> Result<()>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        if transforms.get(handle).is_none() {
            return Err(Error::NonTransformFound);
        }

        unsafe {
            Self::set_world_position_unchecked(nodes, transforms, handle, disp);
            Ok(())
        }
    }

    /// Set position of `Transform` in world space without doing bounds checking.
    pub unsafe fn set_world_position_unchecked<T1, T2, T3>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        disp: T3,
    ) where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let disp = disp.into();
        if nodes.get(handle).is_none() {
            transforms.get_unchecked_mut(handle).set_position(disp);
        } else {
            let mut ancestors_disp = math::Vector3::new(0.0, 0.0, 0.0);
            for v in Node::ancestors(nodes, handle) {
                if let Some(transform) = transforms.get(v) {
                    ancestors_disp += transform.position();
                }
            }

            transforms
                .get_unchecked_mut(handle)
                .set_position(disp - ancestors_disp);
        }
    }

    /// Get position of `Transform` in world space.
    pub fn world_position<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        if transforms.get(handle).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe { Ok(Self::world_position_unchecked(nodes, transforms, handle)) }
        }
    }

    /// Get position of `Transform` in world space without doing bounds checking.
    pub unsafe fn world_position_unchecked<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> math::Vector3<f32>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let transform = transforms.get_unchecked(handle);
        let mut disp = transform.position();
        for v in Node::ancestors(nodes, handle) {
            if let Some(ancestor) = transforms.get(v) {
                disp += ancestor.position();
            }
        }

        disp
    }

    /// Set uniform scale of `Transform` in world space.
    pub fn set_world_scale<T1, T2>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        scale: f32,
    ) -> Result<()>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
    {
        if transforms.get(handle).is_none() {
            return Err(Error::NonTransformFound);
        }

        unsafe {
            Self::set_world_scale_unchecked(nodes, transforms, handle, scale);
            Ok(())
        }
    }

    /// Set uniform scale of `Transform` in world space withoud doing bounds checking.
    pub unsafe fn set_world_scale_unchecked<T1, T2>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        scale: f32,
    ) where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
    {
        if nodes.get(handle).is_none() {
            transforms.get_unchecked_mut(handle).set_scale(scale);
        } else {
            let mut ancestors_scale = 1.0;
            for v in Node::ancestors(nodes, handle) {
                if let Some(transform) = transforms.get(v) {
                    ancestors_scale *= transform.scale();
                }
            }

            if ancestors_scale < ::std::f32::EPSILON {
                transforms.get_unchecked_mut(handle).set_scale(scale);
            } else {
                transforms
                    .get_unchecked_mut(handle)
                    .set_scale(scale / ancestors_scale);
            }
        }
    }

    /// Get the scale of `Transform` in world space.
    pub fn world_scale<T1, T2>(nodes: &T1, transforms: &T2, handle: Entity) -> Result<f32>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        if transforms.get(handle).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe { Ok(Self::world_scale_unchecked(nodes, transforms, handle)) }
        }
    }

    /// Get the scale of `Transform` in world space without doing bounds checking.
    pub unsafe fn world_scale_unchecked<T1, T2>(nodes: &T1, transforms: &T2, handle: Entity) -> f32
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let transform = transforms.get_unchecked(handle);
        let mut scale = transform.scale();
        for v in Node::ancestors(nodes, handle) {
            if let Some(ancestor) = transforms.get(v) {
                scale *= ancestor.scale();
            }
        }
        scale
    }

    /// Set rotation of `Transform` in world space.
    pub fn set_world_rotation<T1, T2, T3>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        rotation: T3,
    ) -> Result<()>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
        T3: Into<math::Quaternion<f32>>,
    {
        if transforms.get(handle).is_none() {
            return Err(Error::NonTransformFound);
        }

        unsafe {
            Self::set_world_rotation_unchecked(nodes, transforms, handle, rotation);
            Ok(())
        }
    }

    /// Set rotation of `Transform` in world space without doing bounds checking.
    pub unsafe fn set_world_rotation_unchecked<T1, T2, T3>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        rotation: T3,
    ) where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
        T3: Into<math::Quaternion<f32>>,
    {
        if nodes.get(handle).is_none() {
            transforms.get_unchecked_mut(handle).set_rotation(rotation);
        } else {
            let mut ancestors_rotation = math::Quaternion::one();
            for v in Node::ancestors(nodes, handle) {
                if let Some(transform) = transforms.get(v) {
                    ancestors_rotation = ancestors_rotation * transform.rotation();
                }
            }

            transforms
                .get_unchecked_mut(handle)
                .set_rotation(rotation.into() * ancestors_rotation.invert());
        }
    }

    /// Get rotation of `Transform` in world space.
    pub fn world_rotation<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Quaternion<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        if transforms.get(handle).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe { Ok(Self::world_rotation_unchecked(nodes, transforms, handle)) }
        }
    }

    /// Get rotation of `Transform` in world space without doing bounds checking.
    pub unsafe fn world_rotation_unchecked<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> math::Quaternion<f32>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let transform = transforms.get_unchecked(handle);
        let mut rotation = transform.rotation();
        for v in Node::ancestors(nodes, handle) {
            if let Some(ancestor) = transforms.get(v) {
                rotation = rotation * ancestor.rotation();
            }
        }

        rotation
    }

    #[allow(dead_code)]
    pub(crate) fn set_world_decomposed<T1, T2>(
        nodes: &T1,
        transforms: &mut T2,
        handle: Entity,
        decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
    ) -> Result<()>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGetMut<Transform>,
    {
        let relative = Transform::world_decomposed(nodes, transforms, handle)?;

        if let Some(inverse) = relative.inverse_transform() {
            unsafe {
                transforms.get_unchecked_mut(handle).decomposed = inverse.concat(&decomposed);
            }
            Ok(())
        } else {
            Err(Error::CanNotInverseTransform)
        }
    }

    pub(crate) fn world_decomposed<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        if transforms.get(handle).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe { Ok(Self::world_decomposed_unchecked(nodes, transforms, handle)) }
        }
    }

    pub(crate) unsafe fn world_decomposed_unchecked<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        let transform = transforms.get_unchecked(handle);
        let mut decomposed = transform.decomposed;
        for v in Node::ancestors(nodes, handle) {
            if let Some(ancestor) = transforms.get(v) {
                decomposed = ancestor.decomposed.concat(&decomposed);
            }
        }
        decomposed
    }
}

impl Transform {
    /// Transforms position from local space to world space.
    pub fn world_transform_point<T1, T2, T3>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        // M = T * R * S
        Ok(decomposed.rot * (v.into() * decomposed.scale) + decomposed.disp)
    }

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    pub fn world_transform_vector<T1, T2, T3>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let decomposed = Transform::world_decomposed(nodes, transforms, handle)?;
        Ok(decomposed.transform_vector(v.into()))
    }

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    pub fn world_transform_direction<T1, T2, T3>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
        v: T3,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
        T3: Into<math::Vector3<f32>>,
    {
        let rotation = Transform::world_rotation(nodes, transforms, handle)?;
        Ok(rotation * v.into())
    }

    /// Return the up direction in world space, which is looking down the positive y-axis.
    pub fn world_up<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        Transform::world_transform_direction(
            nodes,
            transforms,
            handle,
            math::Vector3::new(0.0, 1.0, 0.0),
        )
    }

    /// Return the forward direction in world space, which is looking down the positive z-axis.
    pub fn world_forward<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        Transform::world_transform_direction(
            nodes,
            transforms,
            handle,
            math::Vector3::new(0.0, 0.0, 1.0),
        )
    }

    /// Return the right direction in world space, which is looking down the positive x-axis.
    pub fn world_right<T1, T2>(
        nodes: &T1,
        transforms: &T2,
        handle: Entity,
    ) -> Result<math::Vector3<f32>>
    where
        T1: ArenaGet<Node>,
        T2: ArenaGet<Transform>,
    {
        Transform::world_transform_direction(
            nodes,
            transforms,
            handle,
            math::Vector3::new(1.0, 0.0, 0.0),
        )
    }
}
