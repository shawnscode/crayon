//! A device through which the player views the world.

use math;

/// The projection funcs used when take primitives into camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Orthographic projection with orthographic-size, half vertical
    /// size of camera, in pixels as payload.
    Ortho(f32),
    /// Perspective projection with `fov`, field of view, in degree as payload.
    Perspective(f32),
}

/// A `Camera` is a device through which the player views the world.  We use
/// right-handed coordinates system as default.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    aspect: f32,
    clip: math::Vector2<f32>,
    projection: Projection,
}

impl Camera {
    /// Creates a new camera with perspective projection as default.
    pub fn new() -> Camera {
        Camera {
            aspect: 1.0,
            clip: math::Vector2::new(0.1, 1000.0),
            projection: Projection::Perspective(60.0),
        }
    }

    /// Gets the aspect ratio (width divided by height).
    #[inline(always)]
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Gets the near clipping plane distances.
    #[inline(always)]
    pub fn near_clip_plane(&self) -> f32 {
        self.clip.x
    }

    /// Gets the far clipping plane distances.
    #[inline(always)]
    pub fn far_clip_plane(&self) -> f32 {
        self.clip.y
    }

    /// Sets the near/far clipping plane distances.
    #[inline(always)]
    pub fn set_clip_plane(&mut self, near: f32, far: f32) {
        self.clip = math::Vector2::new(near.min(far), far.max(near));
    }

    /// Gets the projection type and its payload.
    #[inline(always)]
    pub fn projection(&self) -> Projection {
        self.projection
    }

    /// Sets the projection type.
    #[inline(always)]
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    /// Gets the projection matrix based on projector.
    pub fn matrix(&self) -> math::Matrix4<f32> {
        match self.projection {
            Projection::Ortho(vsize) => {
                let hsize = vsize * self.aspect;
                math::ortho(-hsize, hsize, -vsize, vsize, self.clip.x, self.clip.y).into()
            }
            Projection::Perspective(fov) => {
                math::perspective(math::Deg(fov), self.aspect, self.clip.x, self.clip.y).into()
            }
        }
    }
}
