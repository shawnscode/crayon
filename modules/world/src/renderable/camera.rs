//! A device through which the player views the world.

use crayon::math::prelude::*;
use crayon::video::assets::surface::SurfaceHandle;

use spatial::prelude::Transform;

/// A `Camera` is a device through which the player views the world.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    frustum: Frustum<f32>,
    surface: Option<SurfaceHandle>,

    #[doc(hidden)]
    pub(crate) transform: Transform,
}

impl Default for Camera {
    fn default() -> Self {
        let projection = Projection::Perspective {
            fovy: Deg(60.0).into(),
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        };

        Self::new(projection)
    }
}

impl Camera {
    /// Creates a new camera with projection.
    pub fn new(projection: Projection<f32>) -> Self {
        Camera {
            frustum: Frustum::new(projection),
            surface: None,
            transform: Transform::default(),
        }
    }

    /// Creates a new camera with orthographics projection.
    pub fn ortho(w: f32, h: f32, n: f32, f: f32) -> Self {
        let projection = Projection::Ortho {
            width: w,
            height: h,
            near: n,
            far: f,
        };

        Self::new(projection)
    }

    /// Creates a new camera with perspective projection.
    pub fn perspective<T>(fovy: T, aspect: f32, n: f32, f: f32) -> Self
    where
        T: Into<Rad<f32>>,
    {
        let projection = Projection::Perspective {
            fovy: fovy.into(),
            aspect: aspect,
            near: n,
            far: f,
        };

        Self::new(projection)
    }

    /// Sets the drawing surface. If none surface is assigned, the default surface
    /// generated with window framebuffer by the system will be used.
    pub fn set_surface<T>(&mut self, surface: T)
    where
        T: Into<Option<SurfaceHandle>>,
    {
        self.surface = surface.into();
    }

    /// Gets the handle of surface.
    pub fn surface(&self) -> Option<SurfaceHandle> {
        self.surface
    }

    /// Sets the near/far clipping plane distances.
    #[inline]
    pub fn set_clip_plane(&mut self, near: f32, far: f32) {
        let projection = match self.frustum.projection() {
            Projection::Ortho { width, height, .. } => Projection::Ortho {
                width,
                height,
                near,
                far,
            },
            Projection::Perspective { fovy, aspect, .. } => Projection::Perspective {
                fovy,
                aspect,
                near,
                far,
            },
        };

        self.set_projection(projection);
    }

    /// Gets the near clip plane.
    #[inline]
    pub fn near_clip_plane(&self) -> f32 {
        match self.frustum.projection() {
            Projection::Ortho { near, .. } => near,
            Projection::Perspective { near, .. } => near,
        }
    }

    /// Gets the far clip plane.
    #[inline]
    pub fn far_clip_plane(&self) -> f32 {
        match self.frustum.projection() {
            Projection::Ortho { far, .. } => far,
            Projection::Perspective { far, .. } => far,
        }
    }

    /// Gets the projection type and its payload.
    #[inline]
    pub fn projection(&self) -> Projection<f32> {
        self.frustum.projection()
    }

    /// Gets the underlying frustum.
    pub fn frustum(&self) -> Frustum<f32> {
        self.frustum
    }

    /// Sets the projection type.
    #[inline]
    pub fn set_projection(&mut self, projection: Projection<f32>) {
        self.frustum = Frustum::new(projection);
    }
}
