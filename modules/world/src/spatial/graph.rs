use std::iter;

use crayon::prelude::*;
use crayon::utils::hash::{FastHashMap, FastHashSet};
use failure::Error;

use super::node::Node;
use super::transform::Transform;

use Entity;

/// A simple scene graph that used to tore and manipulate the postiion, rotation and scale
/// of the object. We do also keeps a tree relationships betweens object in scene graph, so
/// you can access properties of transformation in both local and world space.
pub struct SceneGraph {
    remap: FastHashMap<Entity, usize>,
    entities: Vec<Entity>,
    nodes: Vec<Node>,
    local_transforms: Vec<Transform>,

    pub(crate) roots: FastHashSet<Entity>,
}

impl SceneGraph {
    pub fn new() -> Self {
        SceneGraph {
            remap: FastHashMap::default(),
            entities: Vec::new(),
            nodes: Vec::new(),
            local_transforms: Vec::new(),
            roots: FastHashSet::default(),
        }
    }

    /// Adds a node.
    pub(crate) fn add(&mut self, ent: Entity) {
        assert!(
            !self.remap.contains_key(&ent),
            "Ent already has components in SceneGraph."
        );

        self.remap.insert(ent, self.entities.len());
        self.entities.push(ent);
        self.nodes.push(Node::default());
        self.local_transforms.push(Transform::default());
        self.roots.insert(ent);
    }

    /// Removes a node and all of its descendants from SceneGraph.
    pub(crate) fn remove(&mut self, ent: Entity) -> Option<Vec<Entity>> {
        if self.remap.contains_key(&ent) {
            self.remove_from_parent(ent, false).unwrap();
            self.roots.remove(&ent);

            let removes: Vec<_> = iter::once(ent).chain(self.descendants(ent)).collect();
            for w in removes.iter() {
                let index = self.remap.remove(w).unwrap();
                self.entities.swap_remove(index);
                self.nodes.swap_remove(index);
                self.local_transforms.swap_remove(index);

                if self.entities.len() != index {
                    *self.remap.get_mut(&self.entities[index]).unwrap() = index;
                }
            }

            Some(removes)
        } else {
            None
        }
    }

    #[inline]
    fn index(&self, ent: Entity) -> Result<usize, Error> {
        self.remap
            .get(&ent)
            .cloned()
            .ok_or_else(|| format_err!("{:?} does not have a node component.", ent))
    }

    #[inline]
    unsafe fn index_unchecked(&self, ent: Entity) -> usize {
        self.remap.get(&ent).cloned().unwrap()
    }
}

impl SceneGraph {
    /// Gets the parent node.
    #[inline]
    pub fn parent(&self, ent: Entity) -> Option<Entity> {
        self.remap.get(&ent).and_then(|v| self.nodes[*v].parent)
    }

    /// Returns ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self, ent: Entity) -> bool {
        self.remap
            .get(&ent)
            .map(|v| self.nodes[*v].first_child.is_none())
            .unwrap_or(false)
    }

    /// Returns ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self, ent: Entity) -> bool {
        self.remap
            .get(&ent)
            .map(|v| self.nodes[*v].parent.is_none())
            .unwrap_or(false)
    }

    /// Attachs a new child to parent transform, before existing children.
    pub fn set_parent<T>(
        &mut self,
        child: Entity,
        parent: T,
        keep_world_pose: bool,
    ) -> Result<(), Error>
    where
        T: Into<Option<Entity>>,
    {
        unsafe {
            let child_index = self.index(child)?;
            let position = if keep_world_pose {
                self.position(child).unwrap()
            } else {
                self.local_transforms[child_index].position
            };

            self.remove_from_parent(child, false)?;

            if let Some(parent) = parent.into() {
                if parent != child {
                    let parent_index = self.index(parent)?;
                    let next_sib = {
                        let node = self.nodes.get_unchecked_mut(parent_index);
                        ::std::mem::replace(&mut node.first_child, Some(child))
                    };

                    let child = self.nodes.get_unchecked_mut(child_index);
                    child.parent = Some(parent);
                    child.next_sib = next_sib;
                } else {
                    bail!("Node can not set self as parent.");
                }

                self.roots.remove(&child);
            }

            if keep_world_pose {
                self.set_position(child, position);
            }

            Ok(())
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(
        &mut self,
        child: Entity,
        keep_world_pose: bool,
    ) -> Result<(), Error> {
        unsafe {
            let child_index = self.index(child)?;
            let position = if keep_world_pose {
                self.position(child).unwrap()
            } else {
                self.local_transforms[child_index].position
            };

            let (parent, next_sib, prev_sib) = {
                let node = self.nodes.get_unchecked_mut(child_index);

                (
                    node.parent.take(),
                    node.next_sib.take(),
                    node.prev_sib.take(),
                )
            };

            if let Some(next_sib) = next_sib {
                let nsi = self.index_unchecked(next_sib);
                self.nodes[nsi].prev_sib = prev_sib;
            }

            if let Some(prev_sib) = prev_sib {
                let psi = self.index_unchecked(prev_sib);
                self.nodes[psi].next_sib = next_sib;
            } else if let Some(parent) = parent {
                // Take this transform as the first child of parent if there is no previous sibling.
                let pi = self.index_unchecked(parent);
                self.nodes[pi].first_child = next_sib;
            }

            self.local_transforms[child_index].position = position;
            self.roots.insert(child);
            Ok(())
        }
    }

    /// Returns an iterator of references to its ancestors.
    #[inline]
    pub fn ancestors(&self, ent: Entity) -> Ancestors {
        Ancestors {
            cursor: self.parent(ent),
            scene: self,
        }
    }

    /// Return true if rhs is one of the ancestor of this `Node`.
    #[inline]
    pub fn is_ancestor(&self, lhs: Entity, rhs: Entity) -> bool {
        for v in self.ancestors(lhs) {
            if v == rhs {
                return true;
            }
        }

        false
    }

    /// Returns an iterator of references to this transform's children.
    #[inline]
    pub fn children(&self, ent: Entity) -> Children {
        let first_child = self
            .remap
            .get(&ent)
            .and_then(|v| self.nodes[*v].first_child);

        Children {
            cursor: first_child,
            scene: self,
        }
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    #[inline]
    pub fn descendants(&self, ent: Entity) -> Descendants {
        let first_child = self
            .remap
            .get(&ent)
            .and_then(|v| self.nodes[*v].first_child);

        Descendants {
            root: ent,
            cursor: first_child,
            scene: self,
        }
    }
}

/// An iterator of references to its ancestors.
pub struct Ancestors<'a> {
    scene: &'a SceneGraph,
    cursor: Option<Entity>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.scene.index_unchecked(ent);
                ::std::mem::replace(&mut self.cursor, self.scene.nodes[index].parent)
            } else {
                None
            }
        }
    }
}

/// An iterator of references to its children.
pub struct Children<'a> {
    scene: &'a SceneGraph,
    cursor: Option<Entity>,
}

impl<'a> Iterator for Children<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.scene.index_unchecked(ent);
                ::std::mem::replace(&mut self.cursor, self.scene.nodes[index].next_sib)
            } else {
                None
            }
        }
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct Descendants<'a> {
    scene: &'a SceneGraph,
    root: Entity,
    cursor: Option<Entity>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.scene.index_unchecked(ent);
                let mut v = *self.scene.nodes.get_unchecked(index);

                // Deep first search when iterating children recursively.
                if v.first_child.is_some() {
                    return ::std::mem::replace(&mut self.cursor, v.first_child);
                }

                if v.next_sib.is_some() {
                    return ::std::mem::replace(&mut self.cursor, v.next_sib);
                }

                // Travel back when we reach leaf-node.
                while let Some(parent) = v.parent {
                    if parent == self.root {
                        break;
                    }

                    let parent_index = self.scene.index_unchecked(parent);
                    v = self.scene.nodes[parent_index];
                    if v.next_sib.is_some() {
                        return ::std::mem::replace(&mut self.cursor, v.next_sib);
                    }
                }
            }

            ::std::mem::replace(&mut self.cursor, None)
        }
    }
}

impl SceneGraph {
    /// Gets the transform in world space.
    #[inline]
    pub fn transform(&self, ent: Entity) -> Option<Transform> {
        self.remap.get(&ent).map(|&index| unsafe {
            self.ancestors(ent)
                .map(|v| self.index_unchecked(v))
                .fold(self.local_transforms[index], |acc, rhs| {
                    self.local_transforms[rhs] * acc
                })
        })
    }

    /// Gets the transform in local space.
    #[inline]
    pub fn local_transform(&self, ent: Entity) -> Option<Transform> {
        self.remap
            .get(&ent)
            .map(|&index| self.local_transforms[index])
    }

    /// Sets the transform in local space.
    #[inline]
    pub fn set_local_transform(&mut self, ent: Entity, transform: Transform) {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index] = transform;
        }
    }
}

impl SceneGraph {
    /// Moves the transform in the direction and distance of translation.
    #[inline]
    pub fn translate<T>(&mut self, ent: Entity, translation: T)
    where
        T: Into<Vector3<f32>>,
    {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index].position += translation.into();
        }
    }

    /// Gets position of the transform in world space.
    #[inline]
    pub fn position(&self, ent: Entity) -> Option<Vector3<f32>> {
        self.transform(ent).map(|transform| transform.position)
    }

    /// Sets position of the transform in world space.
    pub fn set_position<T>(&mut self, ent: Entity, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        if let Some(&index) = self.remap.get(&ent) {
            let t = self
                .parent(ent)
                .map(|v| self.transform(v).unwrap())
                .unwrap_or(Transform::default());

            if let Some(inverse) = t.inverse() {
                self.local_transforms[index].position = inverse.transform_point(position);
            }
        }
    }

    /// Gets position of the transform in local space.
    #[inline]
    pub fn local_position(&self, ent: Entity) -> Option<Vector3<f32>> {
        self.remap
            .get(&ent)
            .map(|&index| self.local_transforms[index].position)
    }

    /// Sets position of the transform in local space.
    #[inline]
    pub fn set_local_position<T>(&mut self, ent: Entity, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index].position = position.into();
        }
    }
}

impl SceneGraph {
    /// Applies a rotation of Entity.
    #[inline]
    pub fn rotate<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index].rotation =
                rotation.into() * self.local_transforms[index].rotation;
        }
    }

    /// Rotate the transform so the forward vector points at target's current position.
    pub fn look_at<T1, T2>(&mut self, ent: Entity, center: T1, up: T2)
    where
        T1: Into<Vector3<f32>>,
        T2: Into<Vector3<f32>>,
    {
        if let Some(eye) = self.position(ent) {
            let center = center.into();
            let up = up.into();

            let dir = (center - eye).normalize();
            let side = up.cross(dir).normalize();
            let up = dir.cross(side).normalize();
            let rotation: Quaternion<f32> = Matrix3::from_cols(side, up, dir).into();

            self.set_rotation(ent, rotation);
        }
    }

    /// Get rotation of the transform in world space.
    #[inline]
    pub fn rotation(&self, ent: Entity) -> Option<Quaternion<f32>> {
        self.remap.get(&ent).map(|&index| unsafe {
            self.ancestors(ent)
                .map(|v| self.index_unchecked(v))
                .fold(self.local_transforms[index].rotation, |acc, rhs| {
                    self.local_transforms[rhs].rotation * acc
                })
        })
    }

    /// Sets rotation of the transform in world space.
    pub fn set_rotation<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        unsafe {
            if let Some(&index) = self.remap.get(&ent) {
                let ancestor_rotation = self
                    .ancestors(ent)
                    .map(|v| self.index_unchecked(v))
                    .fold(Quaternion::one(), |acc, rhs| {
                        self.local_transforms[rhs].rotation * acc
                    });

                self.local_transforms[index].rotation =
                    rotation.into() * ancestor_rotation.invert();
            }
        }
    }

    /// Gets rotation of the transform in local space.
    #[inline]
    pub fn local_rotation(&self, ent: Entity) -> Option<Quaternion<f32>> {
        self.remap
            .get(&ent)
            .map(|&index| self.local_transforms[index].rotation)
    }

    /// Sets rotation of the transform in local space.
    #[inline]
    pub fn set_local_rotation<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index].rotation = rotation.into();
        }
    }
}

impl SceneGraph {
    /// Get scale of the transform in world space.
    #[inline]
    pub fn scale(&self, ent: Entity) -> Option<f32> {
        self.remap.get(&ent).map(|&index| unsafe {
            self.ancestors(ent)
                .map(|v| self.index_unchecked(v))
                .fold(self.local_transforms[index].scale, |acc, rhs| {
                    self.local_transforms[rhs].scale * acc
                })
        })
    }

    /// Sets scale of the transform in world space.
    pub fn set_scale(&mut self, ent: Entity, scale: f32) {
        unsafe {
            if let Some(&index) = self.remap.get(&ent) {
                let ancestor_scale = self
                    .ancestors(ent)
                    .map(|v| self.index_unchecked(v))
                    .fold(1.0, |acc, rhs| self.local_transforms[rhs].scale * acc);

                if ancestor_scale.abs() > ::std::f32::EPSILON {
                    self.local_transforms[index].scale = scale / ancestor_scale;
                } else {
                    self.local_transforms[index].scale = scale;
                }
            }
        }
    }

    /// Gets scale of the transform in local space.
    #[inline]
    pub fn local_scale(&self, ent: Entity) -> Option<f32> {
        self.remap
            .get(&ent)
            .map(|&index| self.local_transforms[index].scale)
    }

    /// Sets scale of the transform in local space.
    #[inline]
    pub fn set_local_scale(&mut self, ent: Entity, scale: f32) {
        if let Some(&index) = self.remap.get(&ent) {
            self.local_transforms[index].scale = scale;
        }
    }
}
