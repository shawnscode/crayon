/// Convenient wrapers to access underlying entities in scene.
use std::any::TypeId;

use crayon::ecs::prelude::*;
use crayon::math;

use components::node::{AncestorsInPlace, ChildrenInPlace, DescendantsInPlace, Node};
use components::transform::Transform;

pub trait EntReader {
    /// The underlying `Handle` of this `Entity`.
    fn id(&self) -> Entity;

    /// Gets a reference to the component corresponding to the `Entity`.
    fn component<T: Component>(&self) -> Option<&T>;

    /// Gets the parent node.
    fn parent(&self) -> Option<Entity>;

    /// Checks if this is the leaf of a hierarchy, aka. has no child.
    fn is_leaf(&self) -> bool;

    /// Checks if this is the root of a hierarchy, aka. has no parent.
    fn is_root(&self) -> bool;

    /// Gets an iterator of references to its ancestors.
    fn ancestors(&self) -> AncestorsInPlace<Fetch<Node>>;

    /// Gets an iterator of references to this node's children.
    fn children(&self) -> ChildrenInPlace<Fetch<Node>>;

    /// Gets an iterator of references to this transform's descendants in tree order.
    fn descendants(&self) -> DescendantsInPlace<Fetch<Node>>;

    /// Checks if a node is one of the ancestor of this node.
    fn is_ancestor(&self, rhs: Entity) -> bool;

    /// Checks if a node is one of the descendants of this node.
    fn is_descendant(&self, rhs: Entity) -> bool;

    /// Gets the scale component in local space.
    fn scale(&self) -> f32;

    /// Gets the scale component in world space.
    fn world_scale(&self) -> f32;

    /// Gets the displacement in local space.
    fn position(&self) -> math::Vector3<f32>;

    /// Gets the displacement in world space.
    fn world_position(&self) -> math::Vector3<f32>;

    /// Gets the rotation in local space.
    fn rotation(&self) -> math::Quaternion<f32>;

    /// Gets the rotation in world space.
    fn world_rotation(&self) -> math::Quaternion<f32>;

    /// Transforms position from local space to world space.
    fn transform_point<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32>;

    /// Transforms vector from local space to world space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    fn transform_vector<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32>;

    /// Transforms direction from local space to world space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    fn transform_direction<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32>;

    /// Return the up direction in world space, which is looking down the positive y-axis.
    fn up(&self) -> math::Vector3<f32>;

    /// Return the forward direction in world space, which is looking down the positive z-axis.
    fn forward(&self) -> math::Vector3<f32>;

    /// Return the right direction in world space, which is looking down the positive x-axis.
    fn right(&self) -> math::Vector3<f32>;
}

pub trait EntWriter: EntReader {
    /// Adds component to entity, returns the old value if exists.
    ///
    /// # Panics
    ///
    /// Panics if trying to add `Node` component or `Transfrom` component.
    fn add<T: Component>(&mut self, value: T) -> Option<T>;

    /// Removes component of entity from the world, and returns it if exists.
    ///
    /// # Panics
    ///
    /// Panics if trying to remove `Node` component or `Transfrom` component.
    fn remove<T: Component>(&mut self) -> Option<T>;

    /// Returns a mutable reference to the componenent corresponding to the `Entity`.
    ///
    /// # Panics
    ///
    /// Panics if trying to get `Node` component or `Transform` component mutablely.
    fn component_mut<T: Component>(&mut self) -> Option<&mut T>;

    /// Attachs self to parent node, before existing children.
    fn set_parent<T1: Into<Option<Entity>>>(&mut self, parent: T1);

    /// Detach a node from its parent and siblings. Children are not affected.
    fn remove_from_parent(&mut self);

    /// Sets the scale component in local space.
    fn set_scale(&mut self, scale: f32);

    /// Sets the scale component in world space.
    fn set_world_scale(&mut self, scale: f32);

    /// Sets the displacement in local space.
    fn set_position<T: Into<math::Vector3<f32>>>(&mut self, position: T);

    /// Sets the displacement in world space.
    fn set_world_position<T: Into<math::Vector3<f32>>>(&mut self, position: T);

    /// Moves this node in the direction and distance of translation.
    fn translate<T: Into<math::Vector3<f32>>>(&mut self, translation: T);

    /// Sets the rotation in local space.
    fn set_rotation<T: Into<math::Quaternion<f32>>>(&mut self, rotation: T);

    /// Sets the rotation in world space.
    fn set_world_rotation<T: Into<math::Quaternion<f32>>>(&mut self, rotation: T);

    /// Applies a rotation in local space.
    fn rotate<T: Into<math::Quaternion<f32>>>(&mut self, rotation: T);

    /// Rotate the transform so the forward vector points at target's current position.
    fn look_at<T1, T2>(&mut self, center: T1, up: T2)
    where
        T1: Into<math::Vector3<f32>>,
        T2: Into<math::Vector3<f32>>;
}

pub struct EntRef<'a> {
    id: Entity,
    world: &'a World,
}

impl<'a> EntRef<'a> {
    pub(crate) fn new(world: &'a World, id: Entity) -> Self {
        EntRef {
            world: world,
            id: id,
        }
    }
}

impl<'a> EntReader for EntRef<'a> {
    #[inline]
    fn id(&self) -> Entity {
        self.id
    }

    #[inline]
    fn component<T: Component>(&self) -> Option<&T> {
        internal::component::<T>(self.world, self.id)
    }

    #[inline]
    fn parent(&self) -> Option<Entity> {
        internal::parent(self.world, self.id)
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        internal::is_leaf(self.world, self.id)
    }

    #[inline]
    fn is_root(&self) -> bool {
        internal::is_root(self.world, self.id)
    }

    #[inline]
    fn ancestors(&self) -> AncestorsInPlace<Fetch<Node>> {
        internal::ancestors(self.world, self.id)
    }

    #[inline]
    fn children(&self) -> ChildrenInPlace<Fetch<Node>> {
        internal::children(self.world, self.id)
    }

    #[inline]
    fn descendants(&self) -> DescendantsInPlace<Fetch<Node>> {
        internal::descendants(self.world, self.id)
    }

    #[inline]
    fn is_ancestor(&self, rhs: Entity) -> bool {
        internal::is_ancestor(self.world, self.id, rhs)
    }

    #[inline]
    fn is_descendant(&self, rhs: Entity) -> bool {
        internal::is_descendant(self.world, self.id, rhs)
    }

    #[inline]
    fn scale(&self) -> f32 {
        internal::scale(self.world, self.id)
    }

    #[inline]
    fn world_scale(&self) -> f32 {
        internal::world_scale(self.world, self.id)
    }

    #[inline]
    fn position(&self) -> math::Vector3<f32> {
        internal::position(self.world, self.id)
    }

    #[inline]
    fn world_position(&self) -> math::Vector3<f32> {
        internal::world_position(self.world, self.id)
    }

    #[inline]
    fn rotation(&self) -> math::Quaternion<f32> {
        internal::rotation(self.world, self.id)
    }

    #[inline]
    fn world_rotation(&self) -> math::Quaternion<f32> {
        internal::world_rotation(self.world, self.id)
    }

    #[inline]
    fn transform_point<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_point(self.world, self.id, v)
    }

    #[inline]
    fn transform_vector<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_vector(self.world, self.id, v)
    }

    #[inline]
    fn transform_direction<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_direction(self.world, self.id, v)
    }

    #[inline]
    fn up(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 1.0, 0.0])
    }

    #[inline]
    fn forward(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 0.0, 1.0])
    }

    #[inline]
    fn right(&self) -> math::Vector3<f32> {
        self.transform_direction([1.0, 0.0, 0.0])
    }
}

pub struct EntRefMut<'a> {
    id: Entity,
    world: &'a mut World,
}

impl<'a> EntRefMut<'a> {
    pub(crate) fn new(world: &'a mut World, id: Entity) -> Self {
        EntRefMut {
            world: world,
            id: id,
        }
    }
}

impl<'a> EntReader for EntRefMut<'a> {
    #[inline]
    fn id(&self) -> Entity {
        self.id
    }

    #[inline]
    fn component<T: Component>(&self) -> Option<&T> {
        internal::component::<T>(self.world, self.id)
    }

    #[inline]
    fn parent(&self) -> Option<Entity> {
        internal::parent(self.world, self.id)
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        internal::is_leaf(self.world, self.id)
    }

    #[inline]
    fn is_root(&self) -> bool {
        internal::is_root(self.world, self.id)
    }

    #[inline]
    fn ancestors(&self) -> AncestorsInPlace<Fetch<Node>> {
        internal::ancestors(self.world, self.id)
    }

    #[inline]
    fn children(&self) -> ChildrenInPlace<Fetch<Node>> {
        internal::children(self.world, self.id)
    }

    #[inline]
    fn descendants(&self) -> DescendantsInPlace<Fetch<Node>> {
        internal::descendants(self.world, self.id)
    }

    #[inline]
    fn is_ancestor(&self, rhs: Entity) -> bool {
        internal::is_ancestor(self.world, self.id, rhs)
    }

    #[inline]
    fn is_descendant(&self, rhs: Entity) -> bool {
        internal::is_descendant(self.world, self.id, rhs)
    }

    #[inline]
    fn scale(&self) -> f32 {
        internal::scale(self.world, self.id)
    }

    #[inline]
    fn world_scale(&self) -> f32 {
        internal::world_scale(self.world, self.id)
    }

    #[inline]
    fn position(&self) -> math::Vector3<f32> {
        internal::position(self.world, self.id)
    }

    #[inline]
    fn world_position(&self) -> math::Vector3<f32> {
        internal::world_position(self.world, self.id)
    }

    #[inline]
    fn rotation(&self) -> math::Quaternion<f32> {
        internal::rotation(self.world, self.id)
    }

    #[inline]
    fn world_rotation(&self) -> math::Quaternion<f32> {
        internal::world_rotation(self.world, self.id)
    }

    #[inline]
    fn transform_point<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_point(self.world, self.id, v)
    }

    #[inline]
    fn transform_vector<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_vector(self.world, self.id, v)
    }

    #[inline]
    fn transform_direction<T1: Into<math::Vector3<f32>>>(&self, v: T1) -> math::Vector3<f32> {
        internal::transform_direction(self.world, self.id, v)
    }

    #[inline]
    fn up(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 1.0, 0.0])
    }

    #[inline]
    fn forward(&self) -> math::Vector3<f32> {
        self.transform_direction([0.0, 0.0, 1.0])
    }

    #[inline]
    fn right(&self) -> math::Vector3<f32> {
        self.transform_direction([1.0, 0.0, 0.0])
    }
}

impl<'a> EntWriter for EntRefMut<'a> {
    #[inline]
    fn add<T: Component>(&mut self, value: T) -> Option<T> {
        internal::add(self.world, self.id, value)
    }

    #[inline]
    fn remove<T: Component>(&mut self) -> Option<T> {
        internal::remove(self.world, self.id)
    }

    #[inline]
    fn component_mut<T: Component>(&mut self) -> Option<&mut T> {
        internal::component_mut(self.world, self.id)
    }

    #[inline]
    fn set_parent<T1>(&mut self, parent: T1)
    where
        T1: Into<Option<Entity>>,
    {
        internal::set_parent(self.world, self.id, parent)
    }

    #[inline]
    fn remove_from_parent(&mut self) {
        internal::remove_from_parent(self.world, self.id)
    }

    #[inline]
    fn set_scale(&mut self, scale: f32) {
        internal::set_scale(self.world, self.id, scale)
    }

    #[inline]
    fn set_world_scale(&mut self, scale: f32) {
        internal::set_world_scale(self.world, self.id, scale)
    }

    #[inline]
    fn set_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        internal::set_position(self.world, self.id, position)
    }

    #[inline]
    fn set_world_position<T>(&mut self, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        internal::set_world_position(self.world, self.id, position)
    }

    #[inline]
    fn translate<T>(&mut self, translation: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        internal::translate(self.world, self.id, translation)
    }

    #[inline]
    fn set_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        internal::set_rotation(self.world, self.id, rotation)
    }

    #[inline]
    fn set_world_rotation<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        internal::set_world_rotation(self.world, self.id, rotation)
    }

    #[inline]
    fn rotate<T>(&mut self, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        internal::rotate(self.world, self.id, rotation)
    }

    #[inline]
    fn look_at<T1, T2>(&mut self, center: T1, up: T2)
    where
        T1: Into<math::Vector3<f32>>,
        T2: Into<math::Vector3<f32>>,
    {
        internal::look_at(self.world, self.id, center, up)
    }
}

mod internal {
    use super::*;
    use crayon::math::InnerSpace;

    #[inline]
    pub fn component<T>(world: &World, id: Entity) -> Option<&T>
    where
        T: Component,
    {
        world.get(id)
    }

    #[inline]
    pub fn parent(world: &World, id: Entity) -> Option<Entity> {
        unsafe {
            let (_, nodes) = world.view_r1::<Node>();
            nodes.get_unchecked(id).parent()
        }
    }

    #[inline]
    pub fn is_leaf(world: &World, id: Entity) -> bool {
        unsafe {
            let (_, nodes) = world.view_r1::<Node>();
            nodes.get_unchecked(id).is_leaf()
        }
    }

    #[inline]
    pub fn is_root(world: &World, id: Entity) -> bool {
        unsafe {
            let (_, nodes) = world.view_r1::<Node>();
            nodes.get_unchecked(id).is_root()
        }
    }

    #[inline]
    pub fn ancestors(world: &World, id: Entity) -> AncestorsInPlace<Fetch<Node>> {
        let (_, nodes) = world.view_r1::<Node>();
        Node::ancestors_in_place(nodes, id)
    }

    #[inline]
    pub fn children(world: &World, id: Entity) -> ChildrenInPlace<Fetch<Node>> {
        let (_, nodes) = world.view_r1::<Node>();
        Node::children_in_place(nodes, id)
    }

    #[inline]
    pub fn descendants(world: &World, id: Entity) -> DescendantsInPlace<Fetch<Node>> {
        let (_, nodes) = world.view_r1::<Node>();
        Node::descendants_in_place(nodes, id)
    }

    #[inline]
    pub fn is_ancestor(world: &World, id: Entity, rhs: Entity) -> bool {
        for v in ancestors(world, id) {
            if v == rhs {
                return true;
            }
        }

        false
    }

    #[inline]
    pub fn is_descendant(world: &World, id: Entity, rhs: Entity) -> bool {
        for v in descendants(world, id) {
            if v == rhs {
                return true;
            }
        }

        false
    }

    #[inline]
    pub fn scale(world: &World, id: Entity) -> f32 {
        unsafe {
            let (_, transforms) = world.view_r1::<Transform>();
            transforms.get_unchecked(id).scale()
        }
    }

    #[inline]
    pub fn world_scale(world: &World, id: Entity) -> f32 {
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            Transform::world_scale_unchecked(&nodes, &transforms, id)
        }
    }

    #[inline]
    pub fn position(world: &World, id: Entity) -> math::Vector3<f32> {
        unsafe {
            let (_, transforms) = world.view_r1::<Transform>();
            transforms.get_unchecked(id).position()
        }
    }

    #[inline]
    pub fn world_position(world: &World, id: Entity) -> math::Vector3<f32> {
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            Transform::world_position_unchecked(&nodes, &transforms, id)
        }
    }

    #[inline]
    pub fn rotation(world: &World, id: Entity) -> math::Quaternion<f32> {
        unsafe {
            let (_, transforms) = world.view_r1::<Transform>();
            transforms.get_unchecked(id).rotation()
        }
    }

    #[inline]
    pub fn world_rotation(world: &World, id: Entity) -> math::Quaternion<f32> {
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            Transform::world_rotation_unchecked(&nodes, &transforms, id)
        }
    }

    #[inline]
    pub fn transform_point<T1>(world: &World, id: Entity, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        // M = T * R * S
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            let decomposed = Transform::world_decomposed_unchecked(&nodes, &transforms, id);
            decomposed.rot * (v.into() * decomposed.scale) + decomposed.disp
        }
    }

    #[inline]
    pub fn transform_vector<T1>(world: &World, id: Entity, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        use crayon::math::Transform as _Transform;
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            let decomposed = Transform::world_decomposed_unchecked(&nodes, &transforms, id);
            decomposed.transform_vector(v.into())
        }
    }

    #[inline]
    pub fn transform_direction<T1>(world: &World, id: Entity, v: T1) -> math::Vector3<f32>
    where
        T1: Into<math::Vector3<f32>>,
    {
        unsafe {
            let (_, nodes, transforms) = world.view_r2();
            let rotation = Transform::world_rotation_unchecked(&nodes, &transforms, id);
            rotation * v.into()
        }
    }

    #[inline]
    pub fn add<T: Component>(world: &mut World, id: Entity, value: T) -> Option<T> {
        let tid = TypeId::of::<T>();
        assert!(tid != TypeId::of::<Node>() && tid != TypeId::of::<Transform>());

        world.add(id, value)
    }

    #[inline]
    pub fn remove<T: Component>(world: &mut World, id: Entity) -> Option<T> {
        let tid = TypeId::of::<T>();
        assert!(tid != TypeId::of::<Node>() && tid != TypeId::of::<Transform>());

        world.remove(id)
    }

    #[inline]
    pub fn component_mut<T: Component>(world: &mut World, id: Entity) -> Option<&mut T> {
        let tid = TypeId::of::<T>();
        assert!(tid != TypeId::of::<Node>() && tid != TypeId::of::<Transform>());

        world.get_mut(id)
    }

    #[inline]
    pub fn set_parent<T1>(world: &mut World, id: Entity, parent: T1)
    where
        T1: Into<Option<Entity>>,
    {
        unsafe {
            let (_, mut nodes) = world.view_w1();
            Node::set_parent_unchecked(&mut nodes, id, parent)
        }
    }

    #[inline]
    pub fn remove_from_parent(world: &mut World, id: Entity) {
        unsafe {
            let (_, mut nodes) = world.view_w1();
            Node::remove_from_parent_unchecked(&mut nodes, id);
        }
    }

    #[inline]
    pub fn set_scale(world: &mut World, id: Entity, scale: f32) {
        unsafe {
            let (_, mut transforms) = world.view_w1::<Transform>();
            transforms.get_unchecked_mut(id).set_scale(scale);
        }
    }

    #[inline]
    pub fn set_world_scale(world: &mut World, id: Entity, scale: f32) {
        unsafe {
            let (_, nodes, mut transforms) = world.view_r1w1();
            Transform::set_world_scale_unchecked(&nodes, &mut transforms, id, scale);
        }
    }

    #[inline]
    pub fn set_position<T>(world: &mut World, id: Entity, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let (_, mut transforms) = world.view_w1::<Transform>();
            transforms.get_unchecked_mut(id).set_position(position);
        }
    }

    #[inline]
    pub fn set_world_position<T>(world: &mut World, id: Entity, position: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let (_, nodes, mut transforms) = world.view_r1w1();
            Transform::set_world_position_unchecked(&nodes, &mut transforms, id, position);
        }
    }

    #[inline]
    pub fn translate<T>(world: &mut World, id: Entity, translation: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        unsafe {
            let (_, mut transforms) = world.view_w1::<Transform>();
            transforms.get_unchecked_mut(id).translate(translation);
        }
    }

    #[inline]
    pub fn set_rotation<T>(world: &mut World, id: Entity, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let (_, mut transforms) = world.view_w1::<Transform>();
            transforms.get_unchecked_mut(id).set_rotation(rotation);
        }
    }

    #[inline]
    pub fn set_world_rotation<T>(world: &mut World, id: Entity, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let (_, nodes, mut transforms) = world.view_r1w1();
            Transform::set_world_rotation_unchecked(&nodes, &mut transforms, id, rotation);
        }
    }

    #[inline]
    pub fn rotate<T>(world: &mut World, id: Entity, rotation: T)
    where
        T: Into<math::Quaternion<f32>>,
    {
        unsafe {
            let (_, mut transforms) = world.view_w1::<Transform>();
            transforms.get_unchecked_mut(id).rotate(rotation);
        }
    }

    #[inline]
    pub fn look_at<T1, T2>(world: &mut World, id: Entity, center: T1, up: T2)
    where
        T1: Into<math::Vector3<f32>>,
        T2: Into<math::Vector3<f32>>,
    {
        unsafe {
            let (_, nodes, mut transforms) = world.view_r1w1();

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
