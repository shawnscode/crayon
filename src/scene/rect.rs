use ecs::VecStorage;
use math;
use super::errors::*;

/// `Rect` is used to store size, anchor information for a 2d rectangle.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    size: math::Vector2<f32>,
    anchor: math::Vector2<f32>,
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            size: math::Vector2::new(0.0, 0.0),
            anchor: math::Vector2::new(0.0, 0.0),
        }
    }
}

/// Declare `Rect` as component with compact vec storage.
declare_component!(Rect, VecStorage);

impl Rect {
    /// Return the calculated rectangle in the local space of the `Transform`.
    pub fn size(&self) -> math::Vector2<f32> {
        self.size
    }

    /// Set the size of `Rect`.
    pub fn set_size(&mut self, size: math::Vector2<f32>) {
        self.size = size;
    }

    /// Return the normalized position, from [0,0] to [1, 1], that it rotates
    /// around.
    pub fn anchor(&self) -> math::Vector2<f32> {
        self.size
    }

    /// Set the normalized anchor position of `Rect`.
    pub fn set_anchor(&mut self, anchor: math::Vector2<f32>) {
        self.anchor[0] = anchor[0].min(1.0).max(0.0);
        self.anchor[1] = anchor[1].min(1.0).max(0.0);
    }
}

impl Rect {
    // /// Returns the corners of the calculated rectangle in the local space of
    // /// its transform.
    // pub fn corners(world: &ecs::World, handle: Entity) -> Result<[math::Vector2; 4]> {}
    // pub fn world_corners();
}