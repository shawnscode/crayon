//! A device through which the player views the world.

// use graphics;
use math;
use graphics::Color;
use ecs::*;

use super::Transform;
use super::errors::*;

/// The projection funcs used when take primitives into camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Orthographic projection with orthographic-size, half vertical
    /// size of camera, in pixels as payload.
    Ortho(f32),
    /// Perspective projection with `fov`, field of view, in degree as payload.
    Perspective(f32),
}

/// A `Camera` is a device through which the player views the world. We use
/// right-handed coordinates system as default.
#[derive(Debug, Clone)]
pub struct Camera {
    aspect: f32,
    clip: (f32, f32),
    projection: Projection,

    // framebuffer: Option<graphics::FrameBufferHandle>,
    // view: Option<graphics::ViewStateHandle>,
    order: u32,
    clear: (Option<Color>, Option<f32>, Option<i32>),
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            aspect: 1.0,
            clip: (0.1, 1000.0),
            projection: Projection::Ortho(128.0),
            order: 0,
            clear: (Some(Color::black()), Some(1.0), None),
            // framebuffer: None,
            // view: None,
        }
    }
}

/// Declare `Camera` as component with hash storage.
declare_component!(Camera, HashMapArena);

impl Camera {
    /// Return the aspect ratio (width divided by height).
    #[inline]
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Set the aspect ratio value.
    #[inline]
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Return the near/far clipping plane distances.
    #[inline]
    pub fn clip_plane(&self) -> (f32, f32) {
        self.clip
    }

    /// Set the near/far clipping plane distances.
    #[inline]
    pub fn set_clip_plane(&mut self, near: f32, far: f32) {
        self.clip = (near.min(far), far.max(near));
    }

    /// Return the projection type and its payload.
    #[inline]
    pub fn projection(&self) -> Projection {
        self.projection
    }

    /// Set the projection type.
    #[inline]
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    // /// Update internal video objects of this camera.
    // #[inline]
    // pub fn update_video_object(&mut self,
    //                            video: &mut graphics::GraphicsSystem)
    //                            -> graphics::errors::Result<()> {
    //     if self.view.is_none() {
    //         self.view = Some(video.create_view(self.framebuffer.clone())?);
    //     }

    //     if let Some(ref vo) = self.view {
    //         vo.object
    //             .write()
    //             .unwrap()
    //             .update_clear(self.clear.0, self.clear.1, self.clear.2);
    //     }

    //     Ok(())
    // }

    // /// Get the handle of view state object.
    // #[inline]
    // pub fn video_object(&self) -> Option<graphics::ViewStateHandle> {
    //     self.view.as_ref().map(|v| v.handle)
    // }

    /// Set clear flags of frame buffer.
    #[inline]
    pub fn set_clear(&mut self, color: Option<Color>, depth: Option<f32>, stencil: Option<i32>) {
        self.clear = (color, depth, stencil);
    }

    // /// Set the render target of this camera. If `FrameBuffer` is none, default
    // /// framebuffer will be used as render target.
    // #[inline]
    // pub fn set_render_target(&mut self, fb: Option<graphics::FrameBufferRef>) {
    //     self.framebuffer = fb;

    //     if let Some(ref v) = self.view {
    //         v.object
    //             .write()
    //             .unwrap()
    //             .update_framebuffer(self.framebuffer.clone());
    //     }
    // }

    // /// Get internal frame buffer, `None` will be returned if default frame-buffer
    // /// is current render target.
    // #[inline]
    // pub fn render_target(&self) -> Option<&graphics::FrameBufferRef> {
    //     self.framebuffer.as_ref()
    // }

    /// Set the rendering order of this camera.
    #[inline]
    pub fn set_render_order(&mut self, order: u32) {
        self.order = order;

        // if let Some(ref v) = self.view {
        //     v.object.write().unwrap().update_order(order);
        // }
    }

    /// Get render order.
    #[inline]
    pub fn render_order(&self) -> u32 {
        self.order
    }
}

impl Camera {
    /// Get the projection matrix based on projector.
    pub fn projection_matrix(arena_mut: &ArenaMutGetter<Camera>,
                             hande: Entity)
                             -> Result<math::Matrix4<f32>> {
        let camera = arena_mut.get(*hande).ok_or(ErrorKind::NonCameraFound)?;
        Ok(match camera.projection {
               Projection::Ortho(vsize) => {
                   let hsize = vsize * camera.aspect;
                   math::ortho(-hsize, hsize, -vsize, vsize, camera.clip.0, camera.clip.1).into()
               }
               Projection::Perspective(fov) => {
                   math::perspective(math::Deg(fov), camera.aspect, camera.clip.0, camera.clip.1)
                       .into()
               }
           })
    }

    /// Get the view matrix of this transform, which is looking down the negative
    /// z-axis.
    pub fn view_matrix(arena_mut: &ArenaMutGetter<Transform>,
                       handle: Entity)
                       -> Result<math::Matrix4<f32>> {
        use math::EuclideanSpace;
        use math::Transform as MathTransform;

        let decomposed = Transform::world_decomposed(&arena_mut, handle)?;
        let dir = math::Vector3::new(0.0, 0.0, -1.0);
        let center = decomposed.transform_point(math::Point3::from_vec(dir));
        let eye = math::Point3::from_vec(decomposed.disp);
        let up = decomposed.rot * math::Vector3::new(0.0, 1.0, 0.0);

        Ok(math::Matrix4::<f32>::look_at(eye, center, up))
    }
}