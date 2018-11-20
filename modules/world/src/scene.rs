//! Scenes contain the environments and menus of your game.

use crayon::errors::Result;
use crayon::math::prelude::{Quaternion, Vector3};
use crayon::utils::prelude::HandlePool;

use assets::prelude::PrefabHandle;
use renderable::prelude::{Camera, Lit, MeshRenderer, Renderable, Renderer};
use spatial::prelude::{SceneGraph, Transform};
use tags::Tags;
use Entity;

/// Scenes contain the environments and menus of your game. Think of each unique
/// Scene as a unique level. In each Scene, you place your environments, obstacles,
/// and decorations, essentially designing and building your game in pieces.
pub struct Scene<R: Renderer> {
    entities: HandlePool<Entity>,
    tags: Tags,

    pub nodes: SceneGraph,
    pub renderables: Renderable,
    pub renderer: R,
}

impl<R: Renderer> Scene<R> {
    pub fn new(renderer: R) -> Self {
        Scene {
            entities: HandlePool::new(),
            tags: Tags::new(),
            nodes: SceneGraph::new(),
            renderables: Renderable::new(),
            renderer: renderer,
        }
    }

    /// Get the length of entitis in this Scene.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Checks if specified `Entity` was created by this scene, and has not been
    /// deleted yet.
    #[inline]
    pub fn contains(&self, ent: Entity) -> bool {
        self.entities.contains(ent)
    }

    /// Create a new Entity.
    #[inline]
    pub fn create<T: AsRef<str>>(&mut self, name: T) -> Entity {
        let e = self.entities.create().into();
        self.nodes.add(e);
        self.tags.add(e, name.as_ref());
        e
    }

    /// Get the name of this Entity.
    #[inline]
    pub fn name(&self, ent: Entity) -> Option<&str> {
        self.tags.name(ent)
    }

    /// Set the name of this Entity.
    #[inline]
    pub fn set_name<T: AsRef<str>>(&mut self, ent: Entity, name: T) {
        self.tags.add(ent, name.as_ref());
    }

    /// Removes a Entity and all of its descendants from this world.
    pub fn delete(&mut self, ent: Entity) -> Option<Vec<Entity>> {
        if let Some(deletions) = self.nodes.remove(ent) {
            for &v in &deletions {
                self.entities.free(v);
                self.tags.remove(v);
                self.renderables.remove_mesh(v);
                self.renderables.remove_lit(v);
                self.renderables.remove_camera(v);
            }

            Some(deletions)
        } else {
            None
        }
    }

    /// Finds a Entity by name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    #[inline]
    pub fn find<N: AsRef<str>>(&self, name: N) -> Option<Entity> {
        let mut components = name.as_ref().trim_left_matches('/').split('/');
        if let Some(first) = components.next() {
            for &v in &self.nodes.roots {
                if let Some(n) = self.tags.name(v) {
                    if n == first {
                        let mut iter = v;
                        while let Some(component) = components.next() {
                            if component == "" {
                                continue;
                            }

                            let mut found = false;
                            for child in self.nodes.children(iter) {
                                if let Some(n) = self.tags.name(child) {
                                    if n == component {
                                        iter = child;
                                        found = true;
                                        break;
                                    }
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
        }

        None
    }

    /// Finds a Entity from specified searching root and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    pub fn find_from<N: AsRef<str>>(&self, root: Entity, name: N) -> Option<Entity> {
        let mut components = name.as_ref().trim_left_matches('/').split('/');
        let mut iter = root;

        while let Some(component) = components.next() {
            if component == "" {
                continue;
            }

            let mut found = false;
            for child in self.nodes.children(iter) {
                if let Some(n) = self.tags.name(child) {
                    if n == component {
                        iter = child;
                        found = true;
                        break;
                    }
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

    /// Instantiates a prefab into entities of this world.
    pub fn instantiate(&mut self, handle: PrefabHandle) -> Result<Entity> {
        if let Some(prefab) = crate::prefab(handle) {
            let mut root = None;
            let mut nodes = Vec::new();
            nodes.push((None, 0));

            while let Some((parent, idx)) = nodes.pop() {
                let n = &prefab.nodes[idx];
                let e = self.create(&n.name);
                self.nodes.set_local_transform(e, n.local_transform);

                if let Some(parent) = parent {
                    self.nodes.set_parent(e, parent, false).unwrap();
                }

                if let Some(mesh) = n.mesh_renderer {
                    let mut mr = MeshRenderer::default();
                    mr.mesh = prefab.meshes[mesh];
                    self.renderables.add_mesh(e, mr);
                }

                if let Some(sib) = n.next_sib {
                    nodes.push((parent, sib));
                }

                if let Some(child) = n.first_child {
                    nodes.push((Some(e), child));
                }

                if root.is_none() {
                    root = Some(e);
                }
            }

            return Ok(root.unwrap());
        } else {
            bail!("{:?} is not valid.", handle);
        }
    }

    /// Draw current scene.
    #[inline]
    pub fn draw(&mut self) {
        self.renderables.draw(&mut self.renderer, &self.nodes);
    }
}

impl<R: Renderer> Scene<R> {
    /// Add camera component to this Entity.
    #[inline]
    pub fn add_camera(&mut self, ent: Entity, camera: Camera) {
        self.renderables.add_camera(ent, camera);
    }

    #[inline]
    pub fn camera(&self, ent: Entity) -> Option<&Camera> {
        self.renderables.camera(ent)
    }

    #[inline]
    pub fn camera_mut(&mut self, ent: Entity) -> Option<&mut Camera> {
        self.renderables.camera_mut(ent)
    }

    /// Remove camera component from this Entity.
    #[inline]
    pub fn remove_camera(&mut self, ent: Entity) {
        self.renderables.remove_camera(ent);
    }

    /// Add light component to this Entity.
    #[inline]
    pub fn add_lit(&mut self, ent: Entity, lit: Lit) {
        self.renderables.add_lit(ent, lit);
    }

    #[inline]
    pub fn lit(&self, ent: Entity) -> Option<&Lit> {
        self.renderables.lit(ent)
    }

    #[inline]
    pub fn lit_mut(&mut self, ent: Entity) -> Option<&mut Lit> {
        self.renderables.lit_mut(ent)
    }

    /// remove light component from this Entity.
    #[inline]
    pub fn remove_lit(&mut self, ent: Entity) {
        self.renderables.remove_lit(ent);
    }

    /// Add mesh component to this Entity.
    #[inline]
    pub fn add_mesh<T: Into<MeshRenderer>>(&mut self, ent: Entity, mesh: T) {
        self.renderables.add_mesh(ent, mesh);
    }

    #[inline]
    pub fn mesh(&self, ent: Entity) -> Option<&MeshRenderer> {
        self.renderables.mesh(ent)
    }

    #[inline]
    pub fn mesh_mut(&mut self, ent: Entity) -> Option<&mut MeshRenderer> {
        self.renderables.mesh_mut(ent)
    }

    /// Remove mesh component from this Entity.
    #[inline]
    pub fn remove_mesh(&mut self, ent: Entity) {
        self.renderables.remove_mesh(ent);
    }

    /// Add material component to this Entity.
    #[inline]
    pub fn add_mtl(&mut self, ent: Entity, mtl: R::Mtl) {
        self.renderer.add_mtl(ent, mtl);
    }

    #[inline]
    pub fn mtl(&self, ent: Entity) -> Option<&R::Mtl> {
        self.renderer.mtl(ent)
    }

    #[inline]
    pub fn mtl_mut(&mut self, ent: Entity) -> Option<&mut R::Mtl> {
        self.renderer.mtl_mut(ent)
    }

    /// Remove material component from this Entity.
    #[inline]
    pub fn remove_mtl(&mut self, ent: Entity) {
        self.renderer.remove_mtl(ent);
    }
}

impl<R: Renderer> Scene<R> {
    /// Gets the parent node.
    #[inline]
    pub fn parent(&self, ent: Entity) -> Option<Entity> {
        self.nodes.parent(ent)
    }

    /// Returns ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self, ent: Entity) -> bool {
        self.nodes.is_leaf(ent)
    }

    /// Returns ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self, ent: Entity) -> bool {
        self.nodes.is_root(ent)
    }

    /// Attachs a new child to parent transform, before existing children.
    pub fn set_parent<T>(&mut self, child: Entity, parent: T, keep_world_pose: bool) -> Result<()>
    where
        T: Into<Option<Entity>>,
    {
        self.nodes.set_parent(child, parent, keep_world_pose)
    }

    /// Detach a transform from its parent and siblings. Children are not affected.
    pub fn remove_from_parent(&mut self, child: Entity, keep_world_pose: bool) -> Result<()> {
        self.nodes.remove_from_parent(child, keep_world_pose)
    }

    /// Returns an iterator of references to its ancestors.
    #[inline]
    pub fn ancestors<'a>(&'a self, ent: Entity) -> impl Iterator<Item = Entity> + 'a {
        self.nodes.ancestors(ent)
    }

    /// Return true if rhs is one of the ancestor of this `Node`.
    #[inline]
    pub fn is_ancestor(&self, lhs: Entity, rhs: Entity) -> bool {
        self.nodes.is_ancestor(lhs, rhs)
    }

    /// Returns an iterator of references to this transform's children.
    #[inline]
    pub fn children<'a>(&'a self, ent: Entity) -> impl Iterator<Item = Entity> + 'a {
        self.nodes.children(ent)
    }

    /// Returns an iterator of references to this transform's descendants in tree order.
    #[inline]
    pub fn descendants<'a>(&'a self, ent: Entity) -> impl Iterator<Item = Entity> + 'a {
        self.nodes.descendants(ent)
    }

    /// Gets the transform in world space.
    #[inline]
    pub fn transform(&self, ent: Entity) -> Option<Transform> {
        self.nodes.transform(ent)
    }

    /// Gets the transform in local space.
    #[inline]
    pub fn local_transform(&self, ent: Entity) -> Option<Transform> {
        self.nodes.local_transform(ent)
    }

    /// Sets the transform in local space.
    #[inline]
    pub fn set_local_transform(&mut self, ent: Entity, transform: Transform) {
        self.nodes.set_local_transform(ent, transform);
    }

    /// Moves the transform in the direction and distance of translation.
    pub fn translate<T>(&mut self, ent: Entity, translation: T)
    where
        T: Into<Vector3<f32>>,
    {
        self.nodes.translate(ent, translation);
    }

    /// Gets position of the transform in world space.
    #[inline]
    pub fn position(&self, ent: Entity) -> Option<Vector3<f32>> {
        self.nodes.position(ent)
    }

    /// Sets position of the transform in world space.
    pub fn set_position<T>(&mut self, ent: Entity, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        self.nodes.set_position(ent, position);
    }

    /// Gets position of the transform in local space.
    #[inline]
    pub fn local_position(&self, ent: Entity) -> Option<Vector3<f32>> {
        self.nodes.local_position(ent)
    }

    /// Sets position of the transform in local space.
    #[inline]
    pub fn set_local_position<T>(&mut self, ent: Entity, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        self.nodes.set_local_position(ent, position);
    }

    /// Applies a rotation of Entity.
    #[inline]
    pub fn rotate<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        self.nodes.rotate(ent, rotation);
    }

    /// Rotate the transform so the forward vector points at target's current position.
    #[inline]
    pub fn look_at<T1, T2>(&mut self, ent: Entity, center: T1, up: T2)
    where
        T1: Into<Vector3<f32>>,
        T2: Into<Vector3<f32>>,
    {
        self.nodes.look_at(ent, center, up);
    }

    /// Get rotation of the transform in world space.
    #[inline]
    pub fn rotation(&self, ent: Entity) -> Option<Quaternion<f32>> {
        self.nodes.rotation(ent)
    }

    /// Sets rotation of the transform in world space.
    pub fn set_rotation<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        self.nodes.set_rotation(ent, rotation);
    }

    /// Gets rotation of the transform in local space.
    #[inline]
    pub fn local_rotation(&self, ent: Entity) -> Option<Quaternion<f32>> {
        self.nodes.local_rotation(ent)
    }

    /// Sets rotation of the transform in local space.
    #[inline]
    pub fn set_local_rotation<T>(&mut self, ent: Entity, rotation: T)
    where
        T: Into<Quaternion<f32>>,
    {
        self.nodes.set_local_rotation(ent, rotation)
    }

    /// Get scale of the transform in world space.
    #[inline]
    pub fn scale(&self, ent: Entity) -> Option<f32> {
        self.nodes.scale(ent)
    }

    /// Sets scale of the transform in world space.
    #[inline]
    pub fn set_scale(&mut self, ent: Entity, scale: f32) {
        self.nodes.set_scale(ent, scale);
    }

    /// Gets scale of the transform in local space.
    #[inline]
    pub fn local_scale(&self, ent: Entity) -> Option<f32> {
        self.nodes.local_scale(ent)
    }

    /// Sets scale of the transform in local space.
    #[inline]
    pub fn set_local_scale(&mut self, ent: Entity, scale: f32) {
        self.nodes.set_local_scale(ent, scale);
    }
}
