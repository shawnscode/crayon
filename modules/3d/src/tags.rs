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
