use crayon::ecs;
use errors::*;

/// `Node` is used to store and manipulate the postiion, rotation and scale
/// of the object. Every `Node` can have a parent, which allows you to apply
/// position, rotation and scale hierarchically.
///
/// `Entity` are used to record the tree relationships. Every access requires going
/// through the arena, which can be cumbersome and comes with some runtime overhead.
/// But it not only keeps code clean and simple, but also makes `Node` could be
/// send or shared across threads safely. This enables e.g. parallel tree traversals.
#[derive(Debug, Clone, Copy)]
pub struct Node {
    parent: Option<ecs::Entity>,
    next_sib: Option<ecs::Entity>,
    prev_sib: Option<ecs::Entity>,
    first_child: Option<ecs::Entity>,
}

/// Declare `Node` as component with compact vec storage.
impl ecs::Component for Node {
    type Arena = ecs::VecArena<Node>;

    // fn drop(arena: &mut Self::Arena, ent: ecs::Entity) {
    //     Node::remove_from_parent(arena, ent);
    // }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            parent: None,
            next_sib: None,
            prev_sib: None,
            first_child: None,
        }
    }
}

impl Node {
    /// Returns the parent widget.
    #[inline]
    pub fn parent(&self) -> Option<ecs::Entity> {
        self.parent
    }

    /// Returns ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.first_child.is_none()
    }

    /// Returns ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

impl Node {
    /// Attachs a new child to parent transform, before existing children.
    pub fn set_parent(arena: &mut ecs::ArenaMut<Node>,
                      child: ecs::Entity,
                      parent: Option<ecs::Entity>)
                      -> Result<()> {
        unsafe {
            if arena.get(child).is_none() {
                bail!(ErrorKind::NonTransformFound);
            }

            // Can not append a transform to it self.
            if let Some(parent) = parent {
                if parent == child || arena.get(parent).is_none() {
                    bail!(ErrorKind::CanNotAttachSelfAsParent);
                }
            }

            Self::remove_from_parent(arena, child)?;

            if let Some(parent) = parent {
                let next_sib = {
                    let node = arena.get_unchecked_mut(parent);
                    ::std::mem::replace(&mut node.first_child, Some(child))
                };

                let child = arena.get_unchecked_mut(child);
                child.parent = Some(parent);
                child.next_sib = next_sib;
            }

            Ok(())
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(arena: &mut ecs::ArenaMut<Node>, handle: ecs::Entity) -> Result<()> {
        unsafe {
            let (parent, next_sib, prev_sib) = {
                if let Some(node) = arena.get_mut(handle) {
                    (node.parent.take(), node.next_sib.take(), node.prev_sib.take())
                } else {
                    bail!(ErrorKind::NonTransformFound);
                }
            };

            if let Some(next_sib) = next_sib {
                arena.get_unchecked_mut(next_sib).prev_sib = prev_sib;
            }

            if let Some(prev_sib) = prev_sib {
                arena.get_unchecked_mut(prev_sib).next_sib = next_sib;
            } else if let Some(parent) = parent {
                // Take this transform as the first child of parent if there is no previous sibling.
                arena.get_unchecked_mut(parent).first_child = next_sib;
            }

            Ok(())
        }
    }

    /// Return an iterator of references to its ancestors.
    pub fn ancestors(arena: &ecs::Arena<Node>, handle: ecs::Entity) -> Ancestors {
        Ancestors {
            arena: arena,
            cursor: arena.get(handle).and_then(|v| v.parent),
        }
    }

    /// Returns an iterator of references to this transform's children.
    pub fn children(arena: &ecs::Arena<Node>, handle: ecs::Entity) -> Children {
        Children {
            arena: arena,
            cursor: arena.get(handle).and_then(|v| v.first_child),
        }
    }


    /// Returns an iterator of references to this transform's descendants in tree order.
    pub fn descendants(arena: &ecs::Arena<Node>, handle: ecs::Entity) -> Descendants {
        Descendants {
            arena: arena,
            root: handle,
            cursor: arena.get(handle).and_then(|v| v.first_child),
        }
    }

    /// Return true if rhs is one of the ancestor of this `Node`.
    pub fn is_ancestor(arena: &ecs::Arena<Node>, lhs: ecs::Entity, rhs: ecs::Entity) -> bool {
        for v in Node::ancestors(arena, lhs) {
            if v == rhs {
                return true;
            }
        }

        false
    }
}

/// An iterator of references to its ancestors.
pub struct Ancestors<'a> {
    arena: &'a ecs::Arena<Node>,
    cursor: Option<ecs::Entity>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = ecs::Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let v = &self.arena.get_unchecked(node);
                return ::std::mem::replace(&mut self.cursor, v.parent);
            }

            None
        }
    }
}

/// An iterator of references to its children.
pub struct Children<'a> {
    arena: &'a ecs::Arena<Node>,
    cursor: Option<ecs::Entity>,
}

impl<'a> Iterator for Children<'a> {
    type Item = ecs::Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let v = &self.arena.get_unchecked(node);
                return ::std::mem::replace(&mut self.cursor, v.next_sib);
            }

            None
        }
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct Descendants<'a> {
    arena: &'a ecs::Arena<Node>,
    root: ecs::Entity,
    cursor: Option<ecs::Entity>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = ecs::Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(node) = self.cursor {
                let mut v = self.arena.get_unchecked(node);

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

                    v = self.arena.get_unchecked(v.parent.unwrap());
                    if v.next_sib.is_some() {
                        return ::std::mem::replace(&mut self.cursor, v.next_sib);
                    }
                }
            }

            return ::std::mem::replace(&mut self.cursor, None);
        }
    }
}