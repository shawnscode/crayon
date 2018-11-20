use crayon::utils::hash::FastHashMap;
use Entity;

pub struct Component<T> {
    pub remap: FastHashMap<Entity, usize>,
    pub entities: Vec<Entity>,
    pub data: Vec<T>,
}

impl<T> Component<T> {
    pub fn new() -> Self {
        Component {
            remap: FastHashMap::default(),
            entities: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, ent: Entity, mut v: T) -> Option<T> {
        if let Some(&index) = self.remap.get(&ent) {
            unsafe {
                ::std::ptr::swap(&mut self.data[index], &mut v);
                Some(v)
            }
        } else {
            self.remap.insert(ent, self.data.len());
            self.entities.push(ent);
            self.data.push(v);
            None
        }
    }

    #[inline]
    pub fn has(&self, ent: Entity) -> bool {
        self.remap.contains_key(&ent)
    }

    pub fn remove(&mut self, ent: Entity) {
        if let Some(index) = self.remap.remove(&ent) {
            self.entities.swap_remove(index);
            self.data.swap_remove(index);

            if self.remap.len() != index {
                *self.remap.get_mut(&self.entities[index]).unwrap() = index;
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
