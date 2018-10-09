use components::utils::Component;
use world::Entity;

pub struct SpriteRenderer {}

pub struct SpriteSystem {
    sprites: Component<Entity, SpriteRenderer>,
}

impl SpriteSystem {
    pub fn add(&mut self, e: Entity) -> Option<SpriteRenderer> {
        self.sprites.add(e, SpriteRenderer {})
    }

    pub fn remove(&mut self, e: Entity) -> Option<SpriteRenderer> {
        self.sprites.remove(e)
    }
}
