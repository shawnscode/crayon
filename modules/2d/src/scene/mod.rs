mod color_transform;
pub use self::color_transform::ColorTransform;

mod node;
pub use self::node::Node;

mod transform;
pub use self::transform::Transform;

use std::iter;

use crayon::utils::{FastHashMap, FastHashSet, HandleLike, VariantStr};

pub struct Scene<H: HandleLike> {
    remap: FastHashMap<H, usize>,
    roots: FastHashSet<H>,

    handles: Vec<H>,
    names: Vec<VariantStr>,
    nodes: Vec<Node<H>>,
    transforms: Vec<Transform>,
    world_transforms: Vec<Transform>,
    color_transforms: Vec<ColorTransform>,
}

impl<H: HandleLike> Scene<H> {
    /// Creates a new and empty `Scene`.
    pub fn new() -> Self {
        Scene {
            remap: FastHashMap::default(),
            roots: FastHashSet::default(),

            handles: Vec::new(),
            names: Vec::new(),
            nodes: Vec::new(),
            transforms: Vec::new(),
            world_transforms: Vec::new(),
            color_transforms: Vec::new(),
        }
    }

    /// Creates a new Entity.
    pub fn add(&mut self, handle: H, name: &str) {
        assert!(
            !self.remap.contains_key(&handle),
            "Handle already has components in this `Scene`."
        );

        self.remap.insert(handle, self.handles.len());
        self.roots.insert(handle);

        self.handles.push(handle);
        self.names.push(name.into());
        self.nodes.push(Node::default());
        self.transforms.push(Transform::default());
        self.world_transforms.push(Transform::default());
        self.color_transforms.push(ColorTransform::default());
    }

    /// Finds a Entity by name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    pub fn find<T: AsRef<str>>(&self, name: T) -> Option<H> {
        unsafe {
            let mut components = name.as_ref().trim_left_matches('/').split('/');
            if let Some(first) = components.next() {
                for &v in &self.roots {
                    if self.names[self.index_unchecked(v)] == first {
                        let mut iter = v;
                        while let Some(component) = components.next() {
                            if component == "" {
                                continue;
                            }

                            let mut found = false;
                            for child in self.children(iter) {
                                if self.names[self.index_unchecked(child)] == component {
                                    iter = child;
                                    found = true;
                                    break;
                                }
                            }

                            if !found {
                                return None;
                            }
                        }

                        while let Some(component) = components.next() {
                            if component == "" {
                                continue;
                            }

                            return None;
                        }

                        return Some(iter);
                    }
                }
            }

            None
        }
    }

    /// Finds a child Entity from `root` by relative name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    pub fn find_from<T: AsRef<str>>(&self, root: H, name: T) -> Option<H> {
        unsafe {
            let mut components = name.as_ref().trim_left_matches('/').split('/');
            let mut iter = root;
            while let Some(component) = components.next() {
                if component == "" {
                    continue;
                }

                let mut found = false;
                for child in self.children(iter) {
                    if self.names[self.index_unchecked(child)] == component {
                        iter = child;
                        found = true;
                        break;
                    }
                }

                if !found {
                    return None;
                }
            }

            while let Some(component) = components.next() {
                if component == "" {
                    continue;
                }

                return None;
            }

            Some(iter)
        }
    }

    /// Removes a `Entity` and all of its descendants from this world.
    pub fn remove(&mut self, handle: H) -> Option<Vec<H>> {
        if !self.remap.contains_key(&handle) {
            return None;
        }

        self.remove_from_parent(handle, false);
        self.roots.remove(&handle);

        let removes: Vec<_> = iter::once(handle).chain(self.descendants(handle)).collect();
        for w in removes.iter() {
            let index = self.remap.remove(w).unwrap();

            self.handles.swap_remove(index);
            self.names.swap_remove(index);
            self.nodes.swap_remove(index);
            self.transforms.swap_remove(index);
            self.world_transforms.swap_remove(index);
            self.color_transforms.swap_remove(index);

            if self.handles.len() != index {
                *self.remap.get_mut(&self.handles[index]).unwrap() = index;
            }
        }

        Some(removes)
    }

    #[inline]
    fn index(&self, handle: H) -> Option<usize> {
        self.remap.get(&handle).cloned()
    }

    #[inline]
    unsafe fn index_unchecked(&self, handle: H) -> usize {
        self.remap.get(&handle).cloned().unwrap()
    }
}

impl<H: HandleLike> Scene<H> {
    /// Gets the parent node.
    #[inline]
    pub fn parent(&self, handle: H) -> Option<H> {
        self.remap.get(&handle).and_then(|v| self.nodes[*v].parent)
    }

    /// Returns ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self, handle: H) -> bool {
        self.remap
            .get(&handle)
            .map(|v| self.nodes[*v].first_child.is_none())
            .unwrap_or(false)
    }

    /// Returns ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self, handle: H) -> bool {
        self.remap
            .get(&handle)
            .map(|v| self.nodes[*v].parent.is_none())
            .unwrap_or(false)
    }

    /// Attachs a new child to parent transform, before existing children.
    pub fn set_parent<T>(&mut self, child: H, parent: T, keep_world_pose: bool)
    where
        T: Into<Option<H>>,
    {
        unsafe {
            if let Some(child_index) = self.index(child) {
                if let Some(parent) = parent.into() {
                    if parent != child {
                        if let Some(parent_index) = self.index(parent) {
                            let transform = if keep_world_pose {
                                self.world_transform(child).unwrap()
                            } else {
                                self.transforms[child_index]
                            };

                            self.remove_from_parent(child, false);
                            self.roots.remove(&child);

                            {
                                let next_sib = {
                                    let node = self.nodes.get_unchecked_mut(parent_index);
                                    ::std::mem::replace(&mut node.first_child, Some(child))
                                };

                                let child_node = self.nodes.get_unchecked_mut(child_index);
                                child_node.parent = Some(parent);
                                child_node.next_sib = next_sib;
                            }

                            if keep_world_pose {
                                self.set_world_transform(child, transform);
                            }

                            return;
                        } else {
                            warn!("The parent node {:?} is not exists in this world.", parent);
                        }
                    }
                }

                self.remove_from_parent(child, true);
            }
        }
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(&mut self, child: H, keep_world_pose: bool) {
        unsafe {
            if let Some(child_index) = self.index(child) {
                let transform = if keep_world_pose {
                    self.world_transform(child).unwrap()
                } else {
                    self.transforms[child_index]
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

                self.transforms[child_index] = transform;
                self.roots.insert(child);
            } else {
                warn!("The {:?} is not exists in this world.", child);
            }
        }
    }

    /// Returns an iterator of references to all the root nodes.
    #[inline]
    pub fn roots<'a>(&'a self) -> impl Iterator<Item = H> + 'a {
        self.roots.iter().cloned()
    }

    /// Returns an iterator of references to its ancestors.
    #[inline]
    pub fn ancestors(&self, handle: H) -> Ancestors<H> {
        Ancestors {
            cursor: self.parent(handle),
            world: self,
        }
    }

    /// Return true if rhs is one of the ancestor of this `Node`.
    #[inline]
    pub fn is_ancestor(&self, lhs: H, rhs: H) -> bool {
        for v in self.ancestors(lhs) {
            if v == rhs {
                return true;
            }
        }

        false
    }

    /// Returns an iterator of references to this transform's children.
    #[inline]
    pub fn children(&self, handle: H) -> Children<H> {
        let first_child = self
            .remap
            .get(&handle)
            .and_then(|v| self.nodes[*v].first_child);

        Children {
            cursor: first_child,
            world: self,
        }
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    #[inline]
    pub fn descendants(&self, handle: H) -> Descendants<H> {
        let first_child = self
            .remap
            .get(&handle)
            .and_then(|v| self.nodes[*v].first_child);

        Descendants {
            root: handle,
            cursor: first_child,
            world: self,
        }
    }
}

/// An iterator of references to its ancestors.
pub struct Ancestors<'a, H: 'a + HandleLike> {
    world: &'a Scene<H>,
    cursor: Option<H>,
}

impl<'a, H: 'a + HandleLike> Iterator for Ancestors<'a, H> {
    type Item = H;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.world.index_unchecked(ent);
                ::std::mem::replace(&mut self.cursor, self.world.nodes[index].parent)
            } else {
                None
            }
        }
    }
}

/// An iterator of references to its children.
pub struct Children<'a, H: 'a + HandleLike> {
    world: &'a Scene<H>,
    cursor: Option<H>,
}

impl<'a, H: 'a + HandleLike> Iterator for Children<'a, H> {
    type Item = H;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.world.index_unchecked(ent);
                ::std::mem::replace(&mut self.cursor, self.world.nodes[index].next_sib)
            } else {
                None
            }
        }
    }
}

/// An iterator of references to its descendants, in tree order.
pub struct Descendants<'a, H: 'a + HandleLike> {
    world: &'a Scene<H>,
    root: H,
    cursor: Option<H>,
}

impl<'a, H: 'a + HandleLike> Iterator for Descendants<'a, H> {
    type Item = H;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(ent) = self.cursor {
                let index = self.world.index_unchecked(ent);
                let mut v = *self.world.nodes.get_unchecked(index);

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

                    let parent_index = self.world.index_unchecked(parent);
                    v = self.world.nodes[parent_index];
                    if v.next_sib.is_some() {
                        return ::std::mem::replace(&mut self.cursor, v.next_sib);
                    }
                }
            }

            ::std::mem::replace(&mut self.cursor, None)
        }
    }
}

impl<H: HandleLike> Scene<H> {
    /// Gets the transform in local space.
    #[inline]
    pub fn transform(&self, handle: H) -> Option<Transform> {
        self.remap.get(&handle).map(|&index| self.transforms[index])
    }

    /// Sets the transform in local space.
    #[inline]
    pub fn set_transform(&mut self, handle: H, transform: Transform) {
        if let Some(&index) = self.remap.get(&handle) {
            self.transforms[index] = transform;
        }
    }

    /// Gets the transform in world space.
    #[inline]
    pub fn world_transform(&self, handle: H) -> Option<Transform> {
        self.remap.get(&handle).map(|&index| unsafe {
            self.ancestors(handle)
                .map(|v| self.index_unchecked(v))
                .fold(self.transforms[index], |acc, rhs| {
                    self.transforms[rhs] * acc
                })
        })
    }

    /// Sets the transform in world space.
    #[inline]
    pub fn set_world_transform(&mut self, handle: H, transform: Transform) {
        if let Some(index) = self.index(handle) {
            let inv = self.nodes[index]
                .parent
                .and_then(|p| self.world_transform(p).and_then(|t| t.inverse()))
                .unwrap_or(Transform::default());

            self.transforms[index] = inv * transform;
        }
    }
}

impl<H: HandleLike> Scene<H> {
    /// Gets the color transform in local space.
    #[inline]
    pub fn color_transform(&self, handle: H) -> Option<ColorTransform> {
        self.remap
            .get(&handle)
            .map(|&index| self.color_transforms[index])
    }

    /// Sets the transform in local space.
    #[inline]
    pub fn set_color_transform(&mut self, handle: H, transform: ColorTransform) {
        if let Some(&index) = self.remap.get(&handle) {
            self.color_transforms[index] = transform;
        }
    }

    /// Gets the transform in world space.
    #[inline]
    pub fn world_color_transform(&self, handle: H) -> Option<ColorTransform> {
        self.remap.get(&handle).map(|&index| unsafe {
            self.ancestors(handle)
                .map(|v| self.index_unchecked(v))
                .fold(self.color_transforms[index], |acc, rhs| {
                    self.color_transforms[rhs] * acc
                })
        })
    }

    /// Sets the transform in world space.
    #[inline]
    pub fn set_world_color_transform(&mut self, handle: H, transform: ColorTransform) {
        if let Some(index) = self.index(handle) {
            let inv = self.nodes[index]
                .parent
                .and_then(|p| self.world_color_transform(p).and_then(|t| t.inverse()))
                .unwrap_or(ColorTransform::default());

            self.color_transforms[index] = inv * transform;
        }
    }
}
