use std::borrow::Borrow;
use ecs;
use ecs::VecArena;
use math;

use super::errors::*;
use super::transform::Transform;

/// `Rect` is used to store size, pivot information for a 2d rectangle. Rotations, size,
/// and scale modifications occur around the pivot so the position of the pivot affects
/// the outcome of a rotation, resizing, or scaling.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    size: math::Vector2<f32>,
    pivot: math::Vector2<f32>,
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            size: math::Vector2::new(0.0, 0.0),
            pivot: math::Vector2::new(0.0, 0.0),
        }
    }
}

/// Declare `Rect` as component with compact vec storage.
declare_component!(Rect, VecArena);

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
    pub fn pivot(&self) -> math::Vector2<f32> {
        self.pivot
    }

    /// Set the normalized pivot position of `Rect`.
    pub fn set_pivot(&mut self, pivot: math::Vector2<f32>) {
        self.pivot[0] = pivot[0].min(1.0).max(0.0);
        self.pivot[1] = pivot[1].min(1.0).max(0.0);
    }

    /// Returns the corners of the calculated rectangle in the local space of
    /// its transform.
    pub fn corners(&self) -> [math::Vector2<f32>; 4] {
        return [math::Vector2::new(-self.pivot[0] * self.size[0], -self.pivot[1] * self.size[1]),
                math::Vector2::new((1.0 - self.pivot[0]) * self.size[0],
                                   -self.pivot[1] * self.size[1]),
                math::Vector2::new((1.0 - self.pivot[0]) * self.size[0],
                                   (1.0 - self.pivot[1]) * self.size[1]),
                math::Vector2::new(-self.pivot[0] * self.size[1],
                                   (1.0 - self.pivot[1]) * self.size[1])];
    }
}

impl Rect {
    /// Returns the corners of the calculated rectangle in the world space of
    /// its transform.
    pub fn world_corners(transforms: &ecs::ArenaMutGetter<Transform>,
                         rects: &ecs::ArenaMutGetter<Rect>,
                         handle: ecs::Entity)
                         -> Result<[math::Vector3<f32>; 4]> {

        let decomposed = Transform::world_decomposed(&transforms, handle)?;
        if let Some(rect) = rects.get(*handle) {
            let corners = rect.corners();

            return Ok([Rect::transform(&decomposed, corners[0]),
                       Rect::transform(&decomposed, corners[1]),
                       Rect::transform(&decomposed, corners[2]),
                       Rect::transform(&decomposed, corners[3])]);
        }

        bail!(ErrorKind::NonTransformFound)
    }

    fn transform(decomposed: &math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,
                 v: math::Vector2<f32>)
                 -> math::Vector3<f32> {
        (decomposed.rot * math::Vector3::<f32>::new(v[0], v[1], 0f32)) + decomposed.disp
    }
}