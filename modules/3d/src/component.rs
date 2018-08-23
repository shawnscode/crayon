use std::collections::HashMap;

use Entity;

pub struct Component<T> {
    remap: HashMap<Entity, usize>,
    entities: Vec<Entity>,
    data: Vec<T>,
}

impl<T> Component<T> {
    pub fn new() -> Self {
        Component {
            remap: HashMap::new(),
            entities: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, ent: Entity, v: T) {
        assert!(
            !self.remap.contains_key(&ent),
            "Ent already has components in Renderer."
        );

        self.remap.insert(ent, self.data.len());
        self.entities.push(ent);
        self.data.push(v);
    }

    pub fn remove(&mut self, ent: Entity) {
        if let Some(v) = self.remap.remove(&ent) {
            self.entities.swap_remove(v);
            self.data.swap_remove(v);

            if self.remap.len() > 0 {
                *self.remap.get_mut(&self.entities[v]).unwrap() = v;
            }
        }
    }

    #[inline]
    pub fn get(&self, ent: Entity) -> Option<&T> {
        let data = &self.data;
        self.remap.get(&ent).map(|&index| &data[index])
    }

    #[inline]
    pub fn get_mut(&mut self, ent: Entity) -> Option<&mut T> {
        let data = &mut self.data;
        self.remap.get(&ent).map(move |&index| &mut data[index])
    }
}
