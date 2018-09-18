use {Component, Entity};

use crayon::utils::VariantStr;

pub struct Tags {
    names: Component<VariantStr>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            names: Component::new(),
        }
    }

    #[inline]
    pub fn add<T: AsRef<str>>(&mut self, ent: Entity, name: T) {
        self.names.add(ent, name.as_ref().into());
    }

    #[inline]
    pub fn remove(&mut self, ent: Entity) {
        self.names.remove(ent);
    }

    #[inline]
    pub fn name(&self, ent: Entity) -> Option<&str> {
        self.names.get(ent).map(|v| v.as_str())
    }
}
