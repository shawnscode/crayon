use std::borrow::Borrow;
use ecs;
use ecs::VecStorage;
use math;

use super::errors::*;
use super::transform::Transform;

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
    pub fn set_size<T>(&mut self, size: T)
        where T: Borrow<math::Vector2<f32>>
    {
        self.size = *size.borrow();
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
    /// Returns the corners of the calculated rectangle in the local space of
    /// its transform.
    pub fn corners(transforms: &ecs::ArenaGetter<Transform>,
                   rects: &ecs::ArenaGetter<Rect>,
                   handle: ecs::Entity)
                   -> Result<[math::Vector2<f32>; 4]> {
        if let Some(transform) = transforms.get(*handle) {
            if let Some(rect) = rects.get(*handle) {
                let disp = transform.position();
                let size = rect.size;
                return Ok([math::Vector2::new(disp[0] - rect.anchor[0] * size[0],
                                              disp[1] - rect.anchor[1] * size[1]),
                           math::Vector2::new(disp[0] + (1.0 - rect.anchor[0]) * size[0],
                                              disp[1] - rect.anchor[1] * size[1]),
                           math::Vector2::new(disp[0] + (1.0 - rect.anchor[0]) * size[0],
                                              disp[1] + (1.0 - rect.anchor[1]) * size[1]),
                           math::Vector2::new(disp[0] - rect.anchor[0] * size[1],
                                              disp[1] + (1.0 - rect.anchor[1]) * size[1])]);
            }
        }

        bail!(ErrorKind::NonTransformFound);
    }

    /// Returns the corners of the calculated rectangle in the world space of
    /// its transform.
    pub fn world_corners(transforms: &ecs::ArenaGetter<Transform>,
                         rects: &ecs::ArenaGetter<Rect>,
                         handle: ecs::Entity)
                         -> Result<[math::Vector2<f32>; 4]> {
        let disp = Transform::world_position(&transforms, handle)?;
        let scale = Transform::world_scale(&transforms, handle)?;

        if let Some(rect) = rects.get(*handle) {
            let size = rect.size * scale;
            return Ok([math::Vector2::new(disp[0] - rect.anchor[0] * size[0],
                                          disp[1] - rect.anchor[1] * size[1]),
                       math::Vector2::new(disp[0] + (1.0 - rect.anchor[0]) * size[0],
                                          disp[1] - rect.anchor[1] * size[1]),
                       math::Vector2::new(disp[0] + (1.0 - rect.anchor[0]) * size[0],
                                          disp[1] + (1.0 - rect.anchor[1]) * size[1]),
                       math::Vector2::new(disp[0] - rect.anchor[0] * size[1],
                                          disp[1] + (1.0 - rect.anchor[1]) * size[1])]);
        }

        bail!(ErrorKind::NonTransformFound)
    }
}