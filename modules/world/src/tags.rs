use inlinable_string::InlinableString;

use utils::prelude::Component;
use Entity;

pub struct Tags {
    names: Component<InlinableString>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            names: Component::new(),
        }
    }

    #[inline]
    pub fn add<T: Into<InlinableString>>(&mut self, ent: Entity, name: T) {
        self.names.add(ent, name.into());
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
