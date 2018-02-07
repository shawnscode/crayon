//! A device through which the player views the world.

use math;
use math::{Angle, Zero};

/// The projection funcs used when take primitives into camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Orthographic projection with orthographic-size, half vertical
    /// size of camera, in pixels as payload.
    Ortho(f32),
    /// Perspective projection with `fov`, field of view, in degree as payload.
    Perspective(math::Rad<f32>),
}

/// A `Camera` is a device through which the player views the world.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    aspect: f32,
    clip: math::Vector2<f32>,
    projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            aspect: 1.0,
            clip: math::Vector2::new(0.1, 1000.0),
            projection: Projection::Perspective(math::Deg(60.0).into()),
        }
    }
}

impl Camera {
    /// Creates a new camera with orthographic projection.
    pub fn ortho(width: f32, height: f32, near: f32, far: f32) -> Camera {
        let camera = Camera {
            aspect: width / height,
            clip: math::Vector2::new(near, far),
            projection: Projection::Ortho(height * 0.5),
        };

        camera.validate();
        camera
    }

    /// Creates a new camera with perspective projection.
    pub fn perspective<T>(fovy: T, aspect: f32, near: f32, far: f32) -> Camera
    where
        T: Into<math::Rad<f32>>,
    {
        let camera = Camera {
            aspect: aspect,
            clip: math::Vector2::new(near, far),
            projection: Projection::Perspective(fovy.into()),
        };

        camera.validate();
        camera
    }

    /// Gets the aspect ratio (width divided by height).
    #[inline]
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Gets the near clipping plane distances.
    #[inline]
    pub fn near_clip_plane(&self) -> f32 {
        self.clip.x
    }

    /// Gets the far clipping plane distances.
    #[inline]
    pub fn far_clip_plane(&self) -> f32 {
        self.clip.y
    }

    /// Sets the near/far clipping plane distances.
    #[inline]
    pub fn set_clip_plane(&mut self, near: f32, far: f32) {
        self.clip = math::Vector2::new(near.min(far), far.max(near));
        self.validate();
    }

    /// Gets the projection type and its payload.
    #[inline]
    pub fn projection(&self) -> Projection {
        self.projection
    }

    /// Sets the projection type.
    #[inline]
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.validate();
    }

    /// Gets the projection matrix based on projector. The camera is aligned along the
    /// local coordinate system's positive z-axis.
    pub fn matrix(&self) -> math::Matrix4<f32> {
        match self.projection {
            Projection::Ortho(vsize) => {
                let hsize = vsize * self.aspect;
                Camera::ortho_matrix(-hsize, hsize, -vsize, vsize, self.clip.x, self.clip.y)
            }
            Projection::Perspective(fovy) => {
                Camera::perspective_matrix(fovy, self.aspect, self.clip.x, self.clip.y)
            }
        }
    }

    /// Gets the orthographic projection matrix.
    pub fn ortho_matrix(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> math::Matrix4<f32> {
        let c0 = [2.0 / (r - l), 0.0, 0.0, 0.0];
        let c1 = [0.0, 2.0 / (t - b), 0.0, 0.0];
        let c2 = [0.0, 0.0, 2.0 / (f - n), 0.0];
        let c3 = [(r + l) / (l - r), (t + b) / (b - t), (f + n) / (n - f), 1.0];
        math::Matrix4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
    }

    /// Gets the perspective projection matrix.
    pub fn perspective_matrix(
        fovy: math::Rad<f32>,
        aspect: f32,
        n: f32,
        f: f32,
    ) -> math::Matrix4<f32> {
        let fc = math::Rad::cot(fovy / 2.0);
        let c0 = [fc / aspect, 0.0, 0.0, 0.0];
        let c1 = [0.0, fc, 0.0, 0.0];
        let c2 = [0.0, 0.0, (f + n) / (f - n), 1.0];
        let c3 = [0.0, 0.0, (2.0 * f * n) / (n - f), 0.0];
        math::Matrix4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
    }

    fn validate(&self) {
        if let Projection::Perspective(fovy) = self.projection {
            assert!(
                fovy > math::Rad::zero(),
                "The vertical field of view cannot be below zero, found: {:?}",
                fovy
            );

            assert!(
                fovy < math::Rad::turn_div_2(),
                "The vertical field of view cannot be greater than a half turn, found: {:?}",
                fovy
            );

            assert!(
                self.aspect > 0.0,
                "The aspect ratio cannot be below zero, found: {:?}",
                self.aspect
            );

            assert!(
                self.clip.x > 0.0,
                "The near plane distance cannot be below zero, found: {:?}",
                self.clip.x
            );

            assert!(
                self.clip.y > 0.0,
                "The far plane distance cannot be below zero, found: {:?}",
                self.clip.y
            );

            assert!(
                self.clip.y > self.clip.x,
                "The far plane cannot be closer than the near plane, found: far: {:?}, near: {:?}",
                self.clip.y,
                self.clip.x
            );
        }
    }
}
