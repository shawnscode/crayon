
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Ord};
use std::sync::Arc;

use core::application;
use ecs;
use graphics;
use resource;

use super::errors::*;
use super::{Transform, Rect, Sprite, Renderable, RenderCamera};

impl_vertex! {
    SpriteVertex {
        position => [Position; Float; 3; false],
        diffuse => [Color0; UByte; 4; true],
        additive => [Color1; UByte; 4; true],
        texcoord => [Texcoord0; Float; 2; false],
    }
}

/// A simple and quick forward sprite renderer with automatic batching.
pub struct SpriteRenderer {
    mat: resource::MaterialPtr,
    vertices: Vec<SpriteVertex>,
}

const MAX_BATCH_VERTICES: usize = 2048;

impl SpriteRenderer {
    pub fn new(application: &mut application::Application) -> Result<Self> {
        Ok(SpriteRenderer {
               mat: resource::factory::material::sprite(&mut application.resources)?,
               vertices: Vec::with_capacity(MAX_BATCH_VERTICES),
           })
    }

    pub fn draw(&mut self,
                mut application: &mut application::Application,
                world: &ecs::World,
                camera: &RenderCamera)
                -> Result<()> {
        let (view, arenas) = world.view_with_3::<Transform, Rect, Sprite>();

        // To batch the sprite vertices, we need to sort the order by the distance
        // from camera.
        let mut sprites = BinaryHeap::new();
        for v in view {
            if arenas.2.get(*v).unwrap().is_visible() {
                let position = Transform::world_position(&arenas.0, v)?;
                let csp = camera.into_view_space(&position);
                if camera.is_inside(&csp) {
                    let zorder = (csp.z.min(camera.clip.0).max(camera.clip.1) * 1000f32) as u32;
                    sprites.push(SpriteOrd {
                                     sprite: v,
                                     zorder: zorder,
                                 });
                }
            }
        }

        // And then we can draw sprites one by one.
        let mut last_mat = None;
        let mut last_texture = None;

        for v in sprites {
            let sprite = arenas.2.get(*v.sprite).unwrap();

            // Commit batched vertices if necessary.
            let mat = sprite.material().map(|v| v.clone());
            let texture = sprite.texture();

            if eq(&last_mat, &mat) || eq(&last_texture, &texture) ||
               self.vertices.len() >= MAX_BATCH_VERTICES {
                self.consume(&mut application, &camera, last_mat, last_texture)?;
            }

            last_mat = mat;
            last_texture = texture;

            let coners = Rect::world_corners(&arenas.0, &arenas.1, v.sprite).unwrap();
            let color = sprite.color().into();
            let additive = sprite.additive_color().into();

            let texcoords = {
                let (position, size) = sprite.texture_rect();
                [[position.0, position.1],
                 [position.0 + size.0, position.1],
                 [position.0 + size.0, position.1 + size.1],
                 [position.0, position.1 + size.1]]
            };

            let v1 = SpriteVertex::new(coners[0].into(), color, additive, texcoords[0]);
            let v2 = SpriteVertex::new(coners[1].into(), color, additive, texcoords[1]);
            let v3 = SpriteVertex::new(coners[2].into(), color, additive, texcoords[2]);
            let v4 = SpriteVertex::new(coners[3].into(), color, additive, texcoords[3]);

            self.vertices.push(v1);
            self.vertices.push(v2);
            self.vertices.push(v4);

            self.vertices.push(v2);
            self.vertices.push(v3);
            self.vertices.push(v4);
        }

        self.consume(&mut application, &camera, last_mat, last_texture)?;
        Ok(())
    }

    fn consume(&mut self,
               mut application: &mut application::Application,
               camera: &RenderCamera,
               mat: Option<resource::MaterialPtr>,
               texture: Option<resource::TexturePtr>)
               -> Result<()> {
        use graphics::UniformVariableType as UVT;

        if self.vertices.len() <= 0 {
            return Ok(());
        }

        let layout = SpriteVertex::layout();
        let vbo =
            application
                .graphics
                .create_vertex_buffer(&layout,
                                      graphics::ResourceHint::Dynamic,
                                      (self.vertices.len() * layout.stride() as usize) as u32,
                                      Some(SpriteVertex::as_bytes(self.vertices.as_slice())))?;

        let mat = mat.unwrap_or(self.mat.clone());
        let mat = mat.write().unwrap();

        let mut uniforms = Vec::new();
        let mut textures = Vec::new();
        mat.build_uniform_variables(&mut application.graphics, &mut textures, &mut uniforms)?;

        if let Some(texture) = texture {
            let mut texture = texture.write().unwrap();
            texture.update_video_object(&mut application.graphics)?;

            if mat.has_uniform_variable("bi_MainTex", UVT::Texture) {
                textures.push(("bi_MainTex", texture.video_object().unwrap()));
            }
        }

        if mat.has_uniform_variable("bi_ViewMatrix", UVT::Matrix4f) {
            uniforms.push(("bi_ViewMatrix", camera.view.into()));
        }

        if mat.has_uniform_variable("bi_ProjectionMatrix", UVT::Matrix4f) {
            uniforms.push(("bi_ProjectionMatrix", camera.projection.into()));
        }

        // println!("Sprite {:#?}", uniforms);

        let pso = {
            let shader = mat.shader();
            let mut shader = shader.write().unwrap();
            shader.update_video_object(&mut application.graphics)?;
            shader.video_object().unwrap()
        };

        application
            .graphics
            .draw(0,
                  camera.vso,
                  pso,
                  &textures,
                  &uniforms,
                  *vbo,
                  None,
                  graphics::Primitive::Triangles,
                  0,
                  self.vertices.len() as u32)?;

        self.vertices.clear();
        Ok(())
    }
}

struct SpriteOrd {
    sprite: ecs::Entity,
    zorder: u32,
}

impl PartialEq for SpriteOrd {
    fn eq(&self, rhs: &SpriteOrd) -> bool {
        self.zorder == rhs.zorder
    }
}

impl Eq for SpriteOrd {}

impl Ord for SpriteOrd {
    fn cmp(&self, rhs: &SpriteOrd) -> Ordering {
        self.partial_cmp(&rhs).unwrap()
    }
}

impl PartialOrd for SpriteOrd {
    fn partial_cmp(&self, rhs: &SpriteOrd) -> Option<Ordering> {
        self.zorder.partial_cmp(&rhs.zorder)
    }
}

fn eq<T>(lhs: &Option<Arc<T>>, rhs: &Option<Arc<T>>) -> bool {
    if (lhs.is_none() && rhs.is_some()) || (lhs.is_some() && rhs.is_none()) {
        return false;
    }

    if lhs.is_none() && rhs.is_none() {
        return true;
    }

    Arc::ptr_eq(lhs.as_ref().unwrap(), rhs.as_ref().unwrap())
}