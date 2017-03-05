use math;
use ecs::HashMapStorage;

/// The projection funcs used when take primitives into camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Orthographic projection with orthographic-size, half vertical
    /// size of camera, in pixels as payload.
    Ortho(f32),
    /// Perspective projection with `fov`, field of view, in degree as payload.
    Perspective(f32),
}

/// A `Camera` is a device through which the player views the world.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    aspect: f32,
    clip: (f32, f32),
    projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            aspect: 1.0,
            clip: (0.1, 100.0),
            projection: Projection::Ortho(128.0),
        }
    }
}

/// Declare `Camera` as component with hash storage.
declare_component!(Camera, HashMapStorage);

impl Camera {
    /// Return the aspect ratio (width divided by height).
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Set the aspect ratio value.
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Return the near/far clipping plane distances.
    pub fn clip_plane(&self) -> (f32, f32) {
        self.clip
    }

    /// Set the near/far clipping plane distances.
    pub fn set_clip_plane(&mut self, near: f32, far: f32) {
        self.clip = (near.min(far), far.max(near));
    }

    /// Return the projection matrix based on projector.
    pub fn projection_matrix(&self) -> math::Matrix4<f32> {
        match self.projection {
            Projection::Ortho(vsize) => {
                let hsize = vsize * self.aspect;
                math::ortho(-hsize, hsize, -vsize, vsize, self.clip.0, self.clip.1).into()
            }
            Projection::Perspective(fov) => {
                math::perspective(math::Deg(fov), self.aspect, self.clip.0, self.clip.1).into()
            }
        }
    }

    /// Return the projection type and its payload.
    pub fn projection(&self) -> Projection {
        self.projection
    }

    /// Set the projection type.
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }
}
