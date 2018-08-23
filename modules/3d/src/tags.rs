use scene::SceneGraph;
use {Component, Entity};

pub struct Tags {
    names: Component<String>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            names: Component::new(),
        }
    }

    #[inline]
    pub fn add<T: AsRef<str>>(&mut self, ent: Entity, name: T) {
        self.names.add(ent, name.as_ref().to_owned());
    }

    #[inline]
    pub fn remove(&mut self, ent: Entity) {
        self.names.remove(ent);
    }

    #[inline]
    pub fn name(&self, ent: Entity) -> Option<&str> {
        self.names.get(ent).map(|v| v.as_ref())
    }
}

/// Finds a Entity by name and returns it.
///
/// If no Entity with name can be found, None is returned. If name contains a '/' character,
/// it traverses the hierarchy like a path name.
pub fn find_by_name<N: AsRef<str>>(scene: &SceneGraph, tags: &Tags, name: N) -> Option<Entity> {
    let mut components = name.as_ref().trim_left_matches('/').split('/');
    if let Some(first) = components.next() {
        for &v in &scene.roots {
            if let Some(n) = tags.name(v) {
                if n == first {
                    let mut iter = v;
                    while let Some(component) = components.next() {
                        if component == "" {
                            continue;
                        }

                        let mut found = false;
                        for child in scene.children(iter) {
                            if let Some(n) = tags.name(child) {
                                if n == component {
                                    iter = child;
                                    found = true;
                                    break;
                                }
                            }
                        }

                        if !found {
                            break;
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
