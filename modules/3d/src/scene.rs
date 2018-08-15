use std::collections::HashMap;

use crayon::ecs::prelude::*;
use crayon::math::{self, One};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Ent({:?}) does not have a node.", _0)]
    NonNodeFound(Entity),
    #[fail(display = "The transform of ent({:?}) can not be inversed.", _0)]
    CanNotInverseTransform(Entity),
    #[fail(display = "Node can not set self as parent.")]
    CanNotAttachSelfAsParent,
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// `Node` is used to store and manipulate the postiion, rotation and scale
/// of the object. Every `Node` can have a parent, which allows you to apply
/// position, rotation and scale hierarchically.
///
/// `Entity` are used to record the tree relationships. Every access requires going
/// through the arena, which can be cumbersome and comes with some runtime overhead.
/// But it not only keeps code clean and simple, but also makes `Node` could be
/// send or shared across threads safely. This enables e.g. parallel tree traversals.
#[derive(Default, Debug, Clone, Copy)]
pub struct Node {
    parent: Option<Entity>,
    next_sib: Option<Entity>,
    prev_sib: Option<Entity>,
    first_child: Option<Entity>,
}

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object. We use a left handed, y-up world coordinate system.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    scale: math::Vector3<f32>,
    translation: math::Vector3<f32>,
    rotation: math::Quaternion<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            scale: math::Vector3::new(1.0, 1.0, 1.0),
            translation: math::Vector3::new(0.0, 0.0, 0.0),
            rotation: math::Quaternion::one(),
        }
    }
}

pub struct SceneGraph {
    remap: HashMap<Entity, usize>,
    entities: Vec<Entity>,
    nodes: Vec<Node>,
    transforms: Vec<Transform>,
    world_transforms: Vec<Transform>,
}

impl SceneGraph {
    pub fn new() -> Self {
        SceneGraph {
            remap: HashMap::new(),
            entities: Vec::new(),
            nodes: Vec::new(),
            transforms: Vec::new(),
            world_transforms: Vec::new(),
        }
    }

    /// Adds a node.
    pub fn add(&mut self, ent: Entity) {
        assert!(
            !self.remap.contains_key(&ent),
            "Ent already has components in SceneGraph."
        );

        self.remap.insert(ent, self.entities.len());
        self.entities.push(ent);
        self.nodes.push(Node::default());
        self.transforms.push(Transform::default());
        self.world_transforms.push(Transform::default());
    }

    /// Removes a node from SceneGraph.
    pub fn remove(&mut self, ent: Entity) {
        if let Some(v) = self.remap.remove(&ent) {
            self.entities.swap_remove(v);
            self.nodes.swap_remove(v);
            self.transforms.swap_remove(v);
            self.world_transforms.swap_remove(v);

            if self.remap.len() > 0 {
                *self.remap.get_mut(&self.entities[v]).unwrap() = v;
            }
        }
    }

    #[inline]
    fn index(&self, ent: Entity) -> Result<usize> {
        self.remap
            .get(&ent)
            .cloned()
            .ok_or(Error::NonNodeFound(ent))
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
    pub fn set_parent<T>(&mut self, child: Entity, parent: T) -> Result<()>
    where
        T: Into<Option<Entity>>,
    {
        self.remove_from_parent(child)?;

        unsafe {
            let v = self.index_unchecked(child);
            if let Some(parent) = parent.into() {
                if parent != child {
                    let w = self.index(parent)?;
                    let next_sib = {
                        let node = self.nodes.get_unchecked_mut(w);
                        ::std::mem::replace(&mut node.first_child, Some(child))
                    };

                    let child = self.nodes.get_unchecked_mut(v);
                    child.parent = Some(parent);
                    child.next_sib = next_sib;
                } else {
                    return Err(Error::CanNotAttachSelfAsParent);
                }
            }

            Ok(())
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(&mut self, child: Entity) -> Result<()> {
        unsafe {
            let (parent, next_sib, prev_sib) = {
                let child_index = self.index(child)?;
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
        let first_child = self.remap
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
        let first_child = self.remap
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

impl SceneGraph {}
