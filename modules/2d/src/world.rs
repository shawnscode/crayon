use std::sync::{Arc, RwLock};

use crayon::utils::HandlePool;

use renderers::Renderer;
use scene::{ColorTransform, Scene, Transform};

impl_handle!(Entity);

pub struct World(RwLock<ExteriorMutableWorld>);

pub trait UpdateVisitor {
    fn update(&mut self, e: Entity, transform: Transform);
}

pub trait RenderVisitor {
    fn render(&mut self, e: Entity, color_transform: ColorTransform);
}

impl World {
    /// Creates a new and empty 2d `World`.
    pub fn new() -> Self {
        World(RwLock::new(ExteriorMutableWorld {
            entities: HandlePool::new(),
            scene: Scene::new(),
        }))
    }

    /// Updates along the hierachy.
    pub fn update<T: FnMut(Entity, Transform)>(&self, mut visitor: T) {
        let world = self.0.read().unwrap();
        for e in world.scene.roots() {
            let transform = world.scene.transform(e).unwrap();
            world.update_recursive(e, transform, &mut visitor);
        }
    }

    /// Renders along the hierachy.
    pub fn render<T: FnMut(Entity, ColorTransform)>(&self, mut visitor: T) {
        let world = self.0.read().unwrap();
        for e in world.scene.roots() {
            let transform = world.scene.color_transform(e).unwrap();
            world.render_recursive(e, transform, &mut visitor);
        }
    }
}

impl World {
    /// Creates a new Entity.
    #[inline]
    pub fn create<'a, T: Into<Option<&'a str>>>(&'a self, name: T) -> Entity {
        self.0.write().unwrap().create(name.into())
    }

    /// Removes a Entity from this world.
    #[inline]
    pub fn delete(&self, e: Entity) {
        self.0.write().unwrap().remove(e);
    }

    /// Finds a Entity by name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    #[inline]
    pub fn find<T: AsRef<str>>(&self, name: T) -> Option<Entity> {
        self.0.read().unwrap().scene.find(name)
    }

    /// Finds a child Entity from `root` by relative name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    pub fn find_from<T: AsRef<str>>(&self, root: Entity, name: T) -> Option<Entity> {
        self.0.read().unwrap().scene.find_from(root, name)
    }

    /// Gets the parent node.
    #[inline]
    pub fn parent(&self, handle: Entity) -> Option<Entity> {
        self.0.read().unwrap().scene.parent(handle)
    }

    /// Attachs a new child to parent transform, before existing children.
    #[inline]
    pub fn set_parent<T>(&self, child: Entity, parent: T, keep_world_pose: bool)
    where
        T: Into<Option<Entity>>,
    {
        let mut world = self.0.write().unwrap();
        world.scene.set_parent(child, parent, keep_world_pose)
    }

    /// Returns ture if this is the leaf of a hierarchy, aka. has no child.
    #[inline]
    pub fn is_leaf(&self, handle: Entity) -> bool {
        let world = self.0.read().unwrap();
        world.scene.is_leaf(handle)
    }

    /// Returns ture if this is the root of a hierarchy, aka. has no parent.
    #[inline]
    pub fn is_root(&self, handle: Entity) -> bool {
        let world = self.0.read().unwrap();
        world.scene.is_root(handle)
    }
}

pub struct ExteriorMutableWorld {
    entities: HandlePool<Entity>,
    scene: Scene<Entity>,
}

impl ExteriorMutableWorld {
    #[inline]
    fn create<T: AsRef<str>>(&mut self, name: Option<T>) -> Entity {
        let e = self.entities.create();
        match name {
            Some(name) => self.scene.add(e, name.as_ref()),
            _ => self.scene.add(e, "entity"),
        }
        e
    }

    #[inline]
    fn remove(&mut self, e: Entity) {
        if self.entities.free(e) {
            self.scene.remove(e);
        }
    }

    fn update_recursive<T: FnMut(Entity, Transform)>(
        &self,
        e: Entity,
        transform: Transform,
        visitor: &mut T,
    ) {
        visitor(e, transform);

        for c in self.scene.children(e) {
            let child_transform = transform * self.scene.transform(c).unwrap();
            self.update_recursive(c, child_transform, visitor);
        }
    }

    fn render_recursive<T: FnMut(Entity, ColorTransform)>(
        &self,
        e: Entity,
        transform: ColorTransform,
        visitor: &mut T,
    ) {
        visitor(e, transform);

        for c in self.scene.children(e) {
            let child_transform = transform * self.scene.color_transform(c).unwrap();
            self.render_recursive(c, child_transform, visitor);
        }
    }
}
