use std::any::TypeId;

use crayon::math;
use crayon::math::InnerSpace;
use crayon::ecs::prelude::*;
use crayon::ecs::cell::{Ref, RefMut};

use components::node::{AncestorsInPlace, ChildrenInPlace, DescendantsInPlace, Node};
use components::transform::Transform;

/// The read-only interfaces of entity in scene.
pub trait EntReader {
    #[doc(hidden)]
    fn world(&self) -> &World;

    /// Gets the handle of this `Entity` in scene.
    fn id(&self) -> Entity;

    /// Returns a reference to the component corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable
    /// borrows can be taken out at the same time.
    fn component<T>(&self) -> Option<Ref<T>>
    where
        T: Component,
    {
        self.world().get(self.id())
    }

    /// Gets the parent node.
    fn parent(&self) -> Option<Entity> {
        unsafe {
            let nodes = self.world().arena::<Node>();
            nodes.get_unchecked(self.id()).parent()
        }
    }

    /// Checks if this is the leaf of a hierarchy, aka. has no child.
    fn is_leaf(&self) -> bool {
        unsafe {
            let nodes = self.world().arena::<Node>();
            nodes.get_unchecked(self.id()).is_leaf()
        }
    }

    /// Checks if this is the root of a hierarchy, aka. has no parent.
    fn is_root(&self) -> bool {
        unsafe {
            let nodes = self.world().arena::<Node>();
            nodes.get_unchecked(self.id()).is_root()
        }
    }

    /// Gets an iterator of references to its ancestors.
    fn ancestors(&self) -> AncestorsInPlace<Fetch<Node>> {
        let nodes = self.world().arena();
        Node::ancestors_in_place(nodes, self.id())
    }

    /// Gets an iterator of references to this node's children.
    fn children(&self) -> ChildrenInPlace<Fetch<Node>> {
        let nodes = self.world().arena();
        Node::children_in_place(nodes, self.id())
    }

    /// Gets an iterator of references to this transform's descendants in tree order.
    fn descendants(&self) -> DescendantsInPlace<Fetch<Node>> {
        let nodes = self.world().arena();
        Node::descendants_in_place(nodes, self.id())
    }

    /// Checks if a node is one of the ancestor of this node.
    fn is_ancestor(&self, rhs: Entity) -> bool {
        for v in self.ancestors() {
            if v == rhs {
                return true;
            }
        }

        false
    }

    /// Checks if a node is one of the descendants of this node.
    fn is_descendant(&self, rhs: Entity) -> bool {
        for v in self.descendants() {
            if v == rhs {
                return true;
            }
        }

        false
    }

    /// Gets the scale component in local space.
    fn scale(&self) -> f32 {
        unsafe {
            let transforms = self.world().arena::<Transform>();
            transforms.get_unchecked(self.id()).scale()
        }
    }

    /// Gets the scale component in world space.
    fn world_scale(&self) -> f32 {
        unsafe {
            let (nodes, transforms) = self.world().arena_r2();
            Transform::world_scale_unchecked(&nodes, &transforms, self.id())
        }
    }

    /// Gets the displacement in local space.
    fn position(&self) -> math::Vector3<f32> {
        unsafe {
            let transforms = self.world().arena::<Transform>();
            transforms.get_unchecked(self.id()).position()
        }
    }

    /// Gets the displacement in world space.
    fn world_position(&self) -> math::Vector3<f32> {
        unsafe {
            let (nodes, transforms) = self.world().arena_r2();
            Transform::world_position_unchecked(&nodes, &transforms, self.id())
        }
    }

    /// Gets the rotation in local space.
    fn rotation(&self) -> math::Quaternion<f32> {
        unsafe {
            let transforms = self.world().arena::<Transform>();
            transforms.get_unchecked(self.id()).rotation()
        }
    }

    /// Gets the rotation in world space.
    fn world_rotation(&self) -> math::Quaternion<f32> {
        unsafe {
            let id = self.id();
            let (nodes, transforms) = self.world().arena_r2();
            Transform::world_rotation_unchecked(&nodes, &transforms, id)
        }
    }

    /// Transforms position from local space to world space.
    fn transform_point<T1>(&self, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        // M = T * R * S
        unsafe {
            let id = self.id();
            let (nodes, transforms) = self.world().arena_r2();
            let decomposed = Transform::world_decomposed_unchecked(&nodes, &transforms, id);
            decomposed.rot * (v.into() * decomposed.scale) + decomposed.disp
        }
    }

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    fn transform_vector<T1>(&self, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        use crayon::math::Transform as _Transform;
        unsafe {
            let id = self.id();
            let (nodes, transforms) = self.world().arena_r2();
            let decomposed = Transform::world_decomposed_unchecked(&nodes, &transforms, id);
            decomposed.transform_vector(v.into())
        }
    }

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    fn transform_direction<T1>(&self, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        unsafe {
            let id = self.id();
            let (nodes, transforms) = self.world().arena_r2();
            let rotation = Transform::world_rotation_unchecked(&nodes, &transforms, id);
            rotation * v.into()
        }
    }

    /// Return the up direction in world space, which is looking down the positive y-axis.
    fn up(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 1.0, 0.0])
    }

    /// Return the forward direction in world space, which is looking down the positive z-axis.
    fn forward(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 0.0, 1.0])
    }

    /// Return the right direction in world space, which is looking down the positive x-axis.
    fn right(&self) -> math::Vector3<f32> {
        self.transform_direction([1.0, 0.0, 0.0])
    }
}

/// The writable interfaces of entity in scene.
pub trait EntWriter: EntReader {
    #[doc(hidden)]
    fn world_mut(&mut self) -> &mut World;

    /// Attachs self to parent node, before existing children.
    fn set_parent<T1>(&mut self, parent: T1)
    where
        T1: Into<Option<Entity>>,
    {
        unsafe {
            let id = self.id();
            let mut nodes = self.world_mut().arena_mut();
            Node::set_parent_unchecked(&mut nodes, id, parent)
        }
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    fn remove_from_parent(&mut self) {
        unsafe {
            let id = self.id();
            let mut nodes = self.world_mut().arena_mut();
            Node::remove_from_parent_unchecked(&mut nodes, id);
        }
    }

    /// Sets the scale component in local space.
    fn set_scale(&mut self, scale: f32) {
        unsafe {
            let id = self.id();
            let mut transforms = self.world_mut().arena_mut::<Transform>();
            transforms.get_unchecked_mut(id).set_scale(scale);
        }
    }

    /// Sets the scale component in world space.
    fn set_world_scale(&mut self, scale: f32) {
        unsafe {
            let id = self.id();
            let (nodes, mut transforms) = self.world_mut().arena_r1w1();
            Transform::set_world_scale_unchecked(&nodes, &mut transforms, id, scale);
        }
    }

    /// Sets the displacement in local space.
    fn set_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let id = self.id();
            let mut transforms = self.world_mut().arena_mut::<Transform>();
            transforms.get_unchecked_mut(id).set_position(position);
        }
    }

    /// Sets the displacement in world space.
    fn set_world_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let id = self.id();
            let (nodes, mut transforms) = self.world_mut().arena_r1w1();
            Transform::set_world_position_unchecked(&nodes, &mut transforms, id, position);
        }
    }

    /// Moves this node in the direction and distance of translation.
    fn translate<T>(&mut self, translation: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let id = self.id();
            let mut transforms = self.world_mut().arena_mut::<Transform>();
            transforms.get_unchecked_mut(id).translate(translation);
        }
    }

    /// Sets the rotation in local space.
    fn set_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let id = self.id();
            let mut transforms = self.world_mut().arena_mut::<Transform>();
            transforms.get_unchecked_mut(id).set_rotation(rotation);
        }
    }

    /// Sets the rotation in world space
    fn set_world_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let id = self.id();
            let (nodes, mut transforms) = self.world_mut().arena_r1w1();
            Transform::set_world_rotation_unchecked(&nodes, &mut transforms, id, rotation);
        }
    }

    /// Applies a rotation in local space.
    fn rotate<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let id = self.id();
            let mut transforms = self.world_mut().arena_mut::<Transform>();
            transforms.get_unchecked_mut(id).rotate(rotation);
        }
    }

    /// Rotate the transform so the forward vector points at target's current position.
    fn look_at<T1, T2>(&mut self, center: T1, up: T2)
    where
        T1: Into<math::Vector3<f32>>,
        T2: Into<math::Vector3<f32>>,
    {
        unsafe {
            let id = self.id();
            let (nodes, mut transforms) = self.world_mut().arena_r1w1();

            let center = center.into();
            let up = up.into();
            let eye = Transform::world_position_unchecked(&nodes, &mut transforms, id);

            let dir = (center - eye).normalize();
            let side = up.cross(dir).normalize();
            let up = dir.cross(side).normalize();
            let rotation: math::Quaternion<f32> = math::Matrix3::from_cols(side, up, dir).into();

            Transform::set_world_rotation_unchecked(&nodes, &mut transforms, id, rotation);
        }
    }
}

/// A simple wrapper of read-only operations of entities.
pub struct EntAccessor<'a> {
    id: Entity,
    world: &'a World,
}

impl<'a> EntReader for EntAccessor<'a> {
    fn world(&self) -> &World {
        self.world
    }

    fn id(&self) -> Entity {
        self.id
    }
}

impl<'a> EntAccessor<'a> {
    pub(crate) fn new(world: &'a World, id: Entity) -> Self {
        EntAccessor {
            world: world,
            id: id,
        }
    }
}

/// A simple wrapper of operations of entities.
pub struct EntAccessorMut<'a> {
    id: Entity,
    world: &'a mut World,
}

impl<'a> EntReader for EntAccessorMut<'a> {
    fn world(&self) -> &World {
        self.world
    }

    fn id(&self) -> Entity {
        self.id
    }
}

impl<'a> EntWriter for EntAccessorMut<'a> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'a> EntAccessorMut<'a> {
    pub(crate) fn new(world: &'a mut World, id: Entity) -> Self {
        EntAccessorMut {
            world: world,
            id: id,
        }
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

        self.world.add(self.id, value)
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

        self.world.remove(self.id)
    }

    /// Returns a mutable reference to the componenent corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    ///
    /// # Panics
    ///
    /// Panics if trying to get `Node` component or `Transform` component mutablely.
    #[inline]
    pub fn component_mut<T>(&mut self) -> Option<RefMut<T>>
    where
        T: Component,
    {
        let id = TypeId::of::<T>();
        assert!(id != TypeId::of::<Node>() && id != TypeId::of::<Transform>());
        self.world.get_mut(self.id)
    }
}
