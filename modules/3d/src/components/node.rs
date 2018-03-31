use crayon::ecs::prelude::*;

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
    parent: Option<Entity>,
    next_sib: Option<Entity>,
    prev_sib: Option<Entity>,
    first_child: Option<Entity>,
}

/// Declare `Node` as component with compact vec storage.
impl Component for Node {
    type Arena = VecArena<Node>;
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
    /// Returns the parent node.
    #[inline]
    pub fn parent(&self) -> Option<Entity> {
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
    pub fn set_parent<T1, T2>(arena: &mut T1, child: Entity, parent: T2) -> Result<()>
    where
        T1: ArenaGetMut<Node>,
        T2: Into<Option<Entity>>,
    {
        if arena.get(child).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe {
                Self::set_parent_unchecked(arena, child, parent);
                Ok(())
            }
        }
    }

    /// Attachs a new child to parent transform, before existing children.
    pub unsafe fn set_parent_unchecked<T1, T2>(arena: &mut T1, child: Entity, parent: T2)
    where
        T1: ArenaGetMut<Node>,
        T2: Into<Option<Entity>>,
    {
        Self::remove_from_parent_unchecked(arena, child);

        let parent = parent.into();
        if let Some(parent) = parent {
            if parent != child {
                let next_sib = {
                    let node = arena.get_unchecked_mut(parent);
                    ::std::mem::replace(&mut node.first_child, Some(child))
                };

                let child = arena.get_unchecked_mut(child);
                child.parent = Some(parent);
                child.next_sib = next_sib;
            }
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent<T1>(arena: &mut T1, handle: Entity) -> Result<()>
    where
        T1: ArenaGetMut<Node>,
    {
        if arena.get(handle).is_none() {
            Err(Error::NonTransformFound)
        } else {
            unsafe {
                Self::remove_from_parent_unchecked(arena, handle);
                Ok(())
            }
        }
    }

    /// Detach a transform from its parent and siblings without doing bounds checking.
    pub unsafe fn remove_from_parent_unchecked<T1>(arena: &mut T1, handle: Entity)
    where
        T1: ArenaGetMut<Node>,
    {
        let (parent, next_sib, prev_sib) = {
            let node = arena.get_unchecked_mut(handle);
            (
                node.parent.take(),
                node.next_sib.take(),
                node.prev_sib.take(),
            )
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
    }

    /// Returns an iterator of references to its ancestors.
    pub fn ancestors<T1>(arena: &T1, handle: Entity) -> Ancestors
    where
        T1: ArenaGet<Node>,
    {
        Ancestors {
            cursor: arena.get(handle).and_then(|v| v.parent),
            arena: arena,
        }
    }

    /// Returns an iterator of references to its ancestors.
    pub fn ancestors_in_place<T1>(arena: T1, handle: Entity) -> AncestorsInPlace<T1>
    where
        T1: ArenaGet<Node>,
    {
        AncestorsInPlace {
            cursor: arena.get(handle).and_then(|v| v.parent),
            arena: arena,
        }
    }

    /// Returns an iterator of references to this transform's children.
    pub fn children<T1>(arena: &T1, handle: Entity) -> Children
    where
        T1: ArenaGet<Node>,
    {
        Children {
            cursor: arena.get(handle).and_then(|v| v.first_child),
            arena: arena,
        }
    }

    /// Returns an iterator of references to this transform's children.
    pub fn children_in_place<T1>(arena: T1, handle: Entity) -> ChildrenInPlace<T1>
    where
        T1: ArenaGet<Node>,
    {
        ChildrenInPlace {
            cursor: arena.get(handle).and_then(|v| v.first_child),
            arena: arena,
        }
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    pub fn descendants<T1>(arena: &T1, handle: Entity) -> Descendants
    where
        T1: ArenaGet<Node>,
    {
        Descendants {
            arena: arena,
            root: handle,
            cursor: arena.get(handle).and_then(|v| v.first_child),
        }
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    pub fn descendants_in_place<T1>(arena: T1, handle: Entity) -> DescendantsInPlace<T1>
    where
        T1: ArenaGet<Node>,
    {
        DescendantsInPlace {
            cursor: arena.get(handle).and_then(|v| v.first_child),
            arena: arena,
            root: handle,
        }
    }

    /// Return true if rhs is one of the ancestor of this `Node`.
    pub fn is_ancestor<T1>(arena: &T1, lhs: Entity, rhs: Entity) -> bool
    where
        T1: ArenaGet<Node>,
    {
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
    arena: &'a ArenaGet<Node>,
    cursor: Option<Entity>,
}

impl<'a> Ancestors<'a> {
    #[inline]
    pub fn next(arena: &ArenaGet<Node>, mut cursor: &mut Option<Entity>) -> Option<Entity> {
        unsafe {
            if let Some(node) = *cursor {
                let v = arena.get_unchecked(node);
                return ::std::mem::replace(&mut cursor, v.parent);
            }

            None
        }
    }
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Ancestors::next(self.arena, &mut self.cursor)
    }
}

/// An iterator of references to its ancestors.
pub struct AncestorsInPlace<T>
where
    T: ArenaGet<Node>,
{
    arena: T,
    cursor: Option<Entity>,
}

impl<T> Iterator for AncestorsInPlace<T>
where
    T: ArenaGet<Node>,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Ancestors::next(&self.arena, &mut self.cursor)
    }
}

/// An iterator of references to its children.
pub struct Children<'a> {
    arena: &'a ArenaGet<Node>,
    cursor: Option<Entity>,
}

impl<'a> Children<'a> {
    #[inline]
    pub fn next(arena: &ArenaGet<Node>, mut cursor: &mut Option<Entity>) -> Option<Entity> {
        unsafe {
            if let Some(node) = *cursor {
                let v = arena.get_unchecked(node);
                return ::std::mem::replace(&mut cursor, v.next_sib);
            }

            None
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Children::next(self.arena, &mut self.cursor)
    }
}

/// An iterator of references to its children.
pub struct ChildrenInPlace<T>
where
    T: ArenaGet<Node>,
{
    arena: T,
    cursor: Option<Entity>,
}

impl<T> Iterator for ChildrenInPlace<T>
where
    T: ArenaGet<Node>,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Children::next(&self.arena, &mut self.cursor)
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct Descendants<'a> {
    arena: &'a ArenaGet<Node>,
    root: Entity,
    cursor: Option<Entity>,
}

impl<'a> Descendants<'a> {
    #[inline]
    pub fn next(
        arena: &ArenaGet<Node>,
        root: Entity,
        mut cursor: &mut Option<Entity>,
    ) -> Option<Entity> {
        unsafe {
            if let Some(node) = *cursor {
                let mut v = *arena.get_unchecked(node);

                // Deep first search when iterating children recursively.
                if v.first_child.is_some() {
                    return ::std::mem::replace(&mut cursor, v.first_child);
                }

                if v.next_sib.is_some() {
                    return ::std::mem::replace(&mut cursor, v.next_sib);
                }

                // Travel back when we reach leaf-node.
                while let Some(parent) = v.parent {
                    if parent == root {
                        break;
                    }

                    v = *arena.get_unchecked(v.parent.unwrap());
                    if v.next_sib.is_some() {
                        return ::std::mem::replace(&mut cursor, v.next_sib);
                    }
                }
            }

            ::std::mem::replace(&mut cursor, None)
        }
    }
}

impl<'a> Iterator for Descendants<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Descendants::next(self.arena, self.root, &mut self.cursor)
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct DescendantsInPlace<T>
where
    T: ArenaGet<Node>,
{
    arena: T,
    root: Entity,
    cursor: Option<Entity>,
}

impl<T> Iterator for DescendantsInPlace<T>
where
    T: ArenaGet<Node>,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        Descendants::next(&self.arena, self.root, &mut self.cursor)
    }
}
