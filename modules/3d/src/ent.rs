use std::any::TypeId;

use crayon::math;
use crayon::ecs::prelude::*;
use crayon::ecs::cell::{Ref, RefMut};

use errors::*;
use components::node::{Ancestors, Children, Descendants, Node};
use components::transform::Transform;

pub struct EntMut<'a> {
    handle: Entity,
    world: &'a mut World,
}

impl<'a> EntMut<'a> {
    pub(crate) fn new(world: &'a mut World, handle: Entity) -> Self {
        EntMut {
            world: world,
            handle: handle,
        }
    }

    /// Gets the handle of this `Entity` in scene.
    #[inline]
    pub fn handle(&self) -> Entity {
        self.handle
    }

    /// Adds component to entity, returns the old value if exists.
    ///
    /// # Panics
    ///
    /// Panics if trying to add `Node` component or `Transfrom` component.
    #[inline]
    pub fn add<T>(&mut self, value: T) -> Option<T>
    where
        T: Component,
    {
        let id = TypeId::of::<T>();
        assert!(id != TypeId::of::<Node>() && id != TypeId::of::<Transform>());

        self.world.add(self.handle, value)
    }

    /// Removes component of entity from the world, and returns it if exists.
    ///
    /// # Panics
    ///
    /// Panics if trying to remove `Node` component or `Transfrom` component.
    #[inline]
    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Component,
    {
        let id = TypeId::of::<T>();
        assert!(id != TypeId::of::<Node>() && id != TypeId::of::<Transform>());

        self.world.remove(self.handle)
    }

    /// Returns a reference to the component corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable
    /// borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the arena is currently mutably borrowed.
    #[inline]
    pub fn component<T>(&self) -> Option<Ref<T>>
    where
        T: Component,
    {
        self.world.get(self.handle)
    }

    /// Returns a mutable reference to the componenent corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    ///
    /// # Panics
    ///
    /// Panics if the arena is currently mutably borrowed.
    /// Panics if trying to get `Node` component or `Transform` component mutablely.
    #[inline]
    pub fn component_mut<T>(&self) -> Option<RefMut<T>>
    where
        T: Component,
    {
        let id = TypeId::of::<T>();
        assert!(id != TypeId::of::<Node>() && id != TypeId::of::<Transform>());

        self.world.get_mut(self.handle)
    }
}

impl<'a> EntMut<'a> {
    /// Gets the parent node.
    #[inline]
    pub fn parent(&self) -> Option<Entity> {
        let nodes = self.world.arena::<Node>();
        unsafe { nodes.get_unchecked(self.handle).parent() }
    }

    /// Checks if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        let nodes = self.world.arena::<Node>();
        unsafe { nodes.get_unchecked(self.handle).is_leaf() }
    }

    /// Checks if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self) -> bool {
        let nodes = self.world.arena::<Node>();
        unsafe { nodes.get_unchecked(self.handle).is_root() }
    }

    /// Attachs self to parent node, before existing children.
    #[inline]
    pub fn set_parent<T1>(&mut self, parent: T1) -> Result<()>
    where
        T1: Into<Option<Entity>>,
    {
        let mut nodes = self.world.arena_mut();
        Node::set_parent(&mut nodes, self.handle, parent)
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    #[inline]
    pub fn remove_from_parent(&mut self) -> Result<()> {
        let mut nodes = self.world.arena_mut();
        Node::remove_from_parent(&mut nodes, self.handle)
    }

    // /// Gets an iterator of references to its ancestors.
    // pub fn ancestors(&self) -> Ancestors {
    //     let nodes = self.world.arena();
    //     Node::ancestors(&nodes, self.handle)
    // }

    // /// Gets an iterator of references to this node's children.
    // pub fn children(&self) -> Children {
    //     let nodes = self.world.arena();
    //     Node::children(&nodes, self.handle)
    // }

    // /// Gets an iterator of references to this transform's descendants in tree order.
    // pub fn descendants(&self) -> Descendants {
    //     let nodes = self.world.arena();
    //     Node::descendants(&nodes, self.handle)
    // }

    // /// Checks if a node is one of the ancestor of this node.
    // pub fn is_ancestor(&self, rhs: Entity) -> bool {
    //     for v in self.ancestors() {
    //         if v == rhs {
    //             return true;
    //         }
    //     }

    //     false
    // }

    // /// Checks if a node is one of the descendants of this node.
    // pub fn is_descendant(&self, rhs: Entity) -> bool {
    //     for v in self.descendants() {
    //         if v == rhs {
    //             return true;
    //         }
    //     }

    //     false
    // }
}

impl<'a> EntMut<'a> {
    /// Gets the scale component in local space.
    #[inline]
    pub fn scale(&self) -> f32 {
        let transforms = self.world.arena::<Transform>();
        unsafe { transforms.get_unchecked(self.handle).scale() }
    }

    /// Gets the scale component in world space.
    #[inline]
    pub fn world_scale(&self) -> Result<f32> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::world_scale(&nodes, &transforms, self.handle)
    }

    /// Sets the scale component in local space.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        let mut transforms = self.world.arena_mut::<Transform>();
        unsafe {
            transforms.get_unchecked_mut(self.handle).set_scale(scale);
        }
    }

    /// Sets the scale component in world space.
    #[inline]
    pub fn set_world_scale(&mut self, scale: f32) -> Result<()> {
        let nodes = self.world.arena();
        let mut transforms = self.world.arena_mut();
        Transform::set_world_scale(&nodes, &mut transforms, self.handle, scale)
    }

    /// Gets the displacement in local space.
    #[inline]
    pub fn position(&self) -> math::Vector3<f32> {
        let transforms = self.world.arena::<Transform>();
        unsafe { transforms.get_unchecked(self.handle).position() }
    }

    /// Gets the displacement in world space.
    #[inline]
    pub fn world_position(&self) -> Result<math::Vector3<f32>> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::world_position(&nodes, &transforms, self.handle)
    }

    /// Sets the displacement in local space.
    #[inline]
    pub fn set_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        let mut transforms = self.world.arena_mut::<Transform>();
        unsafe {
            transforms
                .get_unchecked_mut(self.handle)
                .set_position(position);
        }
    }

    /// Sets the displacement in world space.
    #[inline]
    pub fn set_world_position<T>(&mut self, position: T) -> Result<()>
    where
        T: Into<math::Vector3<f32>>,
    {
        let nodes = self.world.arena();
        let mut transforms = self.world.arena_mut();
        Transform::set_world_position(&nodes, &mut transforms, self.handle, position)
    }

    /// Moves this node in the direction and distance of translation.
    #[inline]
    pub fn translate<T>(&mut self, translation: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        let mut transforms = self.world.arena_mut::<Transform>();
        unsafe {
            transforms
                .get_unchecked_mut(self.handle)
                .translate(translation);
        }
    }

    /// Gets the rotation in local space.
    #[inline]
    pub fn rotation(&self) -> math::Quaternion<f32> {
        let transforms = self.world.arena::<Transform>();
        unsafe { transforms.get_unchecked(self.handle).rotation() }
    }

    /// Gets the rotation in world space.
    #[inline]
    pub fn world_rotation(&self) -> Result<math::Quaternion<f32>> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::world_rotation(&nodes, &transforms, self.handle)
    }

    /// Sets the rotation in local space.
    #[inline]
    pub fn set_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        let mut transforms = self.world.arena_mut::<Transform>();
        unsafe {
            transforms
                .get_unchecked_mut(self.handle)
                .set_rotation(rotation);
        }
    }

    /// Sets the rotation in world space
    #[inline]
    pub fn set_world_rotation<T>(&mut self, rotation: T) -> Result<()>
    where
        T: Into<math::Quaternion<f32>>,
    {
        let nodes = self.world.arena();
        let mut transforms = self.world.arena_mut();
        Transform::set_world_rotation(&nodes, &mut transforms, self.handle, rotation)
    }

    /// Applies a rotation in local space.
    #[inline]
    pub fn rotate<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        let mut transforms = self.world.arena_mut::<Transform>();
        unsafe {
            transforms.get_unchecked_mut(self.handle).rotate(rotation);
        }
    }

    /// Rotate the transform so the forward vector points at target's current position.
    #[inline]
    pub fn look_at<T1, T2>(&mut self, dst: T1, up: T2) -> Result<()>
    where
        T1: Into<math::Vector3<f32>>,
        T2: Into<math::Vector3<f32>>,
    {
        let nodes = self.world.arena();
        let mut transforms = self.world.arena_mut();
        Transform::look_at(&nodes, &mut transforms, self.handle, dst, up)
    }

    /// Transforms position from local space to world space.
    #[inline]
    pub fn transform_point<T1>(&self, v: T1) -> Result<math::Vector3<f32>>
    where
        T1: Into<math::Vector3<f32>>,
    {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::transform_point(&nodes, &transforms, self.handle, v)
    }

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    #[inline]
    pub fn transform_vector<T1>(&self, v: T1) -> Result<math::Vector3<f32>>
    where
        T1: Into<math::Vector3<f32>>,
    {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::transform_vector(&nodes, &transforms, self.handle, v)
    }

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    #[inline]
    pub fn transform_direction<T1>(&self, v: T1) -> Result<math::Vector3<f32>>
    where
        T1: Into<math::Vector3<f32>>,
    {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::transform_direction(&nodes, &transforms, self.handle, v)
    }

    /// Return the up direction in world space, which is looking down the positive y-axis.
    #[inline]
    pub fn up(&self) -> Result<math::Vector3<f32>> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::up(&nodes, &transforms, self.handle)
    }

    /// Return the forward direction in world space, which is looking down the positive z-axis.
    #[inline]
    pub fn forward(&self) -> Result<math::Vector3<f32>> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::forward(&nodes, &transforms, self.handle)
    }

    /// Return the right direction in world space, which is looking down the positive x-axis.
    #[inline]
    pub fn right(&self) -> Result<math::Vector3<f32>> {
        let nodes = self.world.arena();
        let transforms = self.world.arena();
        Transform::right(&nodes, &transforms, self.handle)
    }
}
