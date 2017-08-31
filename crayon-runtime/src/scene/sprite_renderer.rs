
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Ord};
use std::sync::RwLock;

use core::application;
use ecs;
use graphics;
use math;
use resource;

use super::errors::*;
use super::{Transform, Rect, Camera, Sprite, Renderable};

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
    pipeline: graphics::PipelineStateRef,
    vertices: Vec<SpriteVertex>,
}

const SPRITE_VS: &'static str = include_str!("resources/sprite.vs");
const SPRITE_FS: &'static str = include_str!("resources/sprite.fs");
const MAX_BATCH_VERTICES: usize = 1024;

impl SpriteRenderer {
    pub fn new(application: &mut application::Application) -> Result<Self> {
        let attributes = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 3)
            .with(graphics::VertexAttribute::Color0, 4)
            .with(graphics::VertexAttribute::Color1, 4)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .finish();

        let mut state = graphics::RenderState::default();
        {
            // Enable color blend with equation: src * srcAlpha + dest * (1-srcAlpha);
            use graphics::{Equation, BlendFactor, BlendValue};
            state.color_blend = Some((Equation::Add,
                                      BlendFactor::Value(BlendValue::SourceAlpha),
                                      BlendFactor::OneMinusValue(BlendValue::SourceAlpha)));
        }

        let pipeline = application
            .graphics
            .create_pipeline(SPRITE_VS, SPRITE_FS, &state, &attributes)?;

        Ok(SpriteRenderer {
               pipeline: pipeline,
               vertices: Vec::with_capacity(MAX_BATCH_VERTICES),
           })
    }

    pub fn render(&mut self,
                  mut application: &mut application::Application,
                  world: &ecs::World,
                  camera: ecs::Entity)
                  -> Result<()> {
        // Parse the essential matrixs from camera.
        if !world.has::<Transform>(camera) || !world.has::<Camera>(camera) {
            bail!(ErrorKind::CanNotDrawWithoutCamera);
        }

        let view_mat = {
            let arena = world.arena::<Transform>().unwrap();
            Transform::view(&arena, camera)?
        };

        let (vso, proj_mat) = {
            let mut camera = world.fetch_mut::<Camera>(camera).unwrap();
            camera.update_video_object(&mut application.graphics)?;
            (camera.video_object().unwrap(), camera.projection_matrix())
        };

        let (view, arenas) = world.view_with_3::<Transform, Rect, Sprite>();

        // To batch the sprite vertices, we need to sort the order by the distance
        // from camera.
        let mut sprites = BinaryHeap::new();
        for v in view {
            if arenas.2.get(*v).unwrap().visible() {
                let position = Transform::world_position(&arenas.0, v).unwrap();
                sprites.push(SpriteOrd {
                                 sprite: v,
                                 priority: position[2],
                             });
            }
        }

        // And then we can draw sprites one by one.
        let mut last_texture = None;
        for v in sprites {
            let sprite = arenas.2.get(*v.sprite).unwrap();

            // Commit batched vertices if necessary.
            let texture = sprite.texture();
            if !eq(last_texture, texture) || self.vertices.len() >= MAX_BATCH_VERTICES {
                self.consume(&mut application, &vso, &view_mat, &proj_mat, last_texture)?;
            }

            last_texture = texture;

            let coners = Rect::world_corners(&arenas.0, &arenas.1, v.sprite).unwrap();
            let color = sprite.color().into();
            let additive = sprite.additive_color().into();
            let (position, size) = sprite.texture_rect();

            let v1 = SpriteVertex::new(coners[0].into(), color, additive, [position.0, position.1]);

            let v2 = SpriteVertex::new(coners[1].into(),
                                       color,
                                       additive,
                                       [position.0 + size.0, position.1]);

            let v3 = SpriteVertex::new(coners[2].into(),
                                       color,
                                       additive,
                                       [position.0 + size.0, position.1 + size.1]);

            let v4 = SpriteVertex::new(coners[3].into(),
                                       color,
                                       additive,
                                       [position.0, position.1 + size.1]);

            self.vertices.push(v1);
            self.vertices.push(v2);
            self.vertices.push(v4);

            self.vertices.push(v2);
            self.vertices.push(v3);
            self.vertices.push(v4);
        }

        self.consume(&mut application, &vso, &view_mat, &proj_mat, last_texture)?;
        Ok(())
    }

    fn consume(&mut self,
               mut application: &mut application::Application,
               vso: &graphics::ViewHandle,
               view_mat: &math::Matrix4<f32>,
               proj_mat: &math::Matrix4<f32>,
               texture: Option<&resource::TexturePtr>)
               -> Result<()> {
        if self.vertices.len() <= 0 {
            return Ok(());
        }

        let uniforms = [("u_View", graphics::UniformVariable::Matrix4f(*view_mat.as_ref(), true)),
                        ("u_Proj", graphics::UniformVariable::Matrix4f(*proj_mat.as_ref(), true))];
        let layout = SpriteVertex::layout();
        let vbo =
            application
                .graphics
                .create_vertex_buffer(&layout,
                                      graphics::ResourceHint::Dynamic,
                                      (self.vertices.len() * layout.stride() as usize) as u32,
                                      Some(SpriteVertex::as_bytes(self.vertices.as_slice())))?;

        let mut textures = Vec::new();
        if let Some(texture) = texture {
            let mut locked_texture = texture.write().unwrap();
            locked_texture
                .update_video_object(&mut application.graphics)?;

            if let Some(texture_handle) = locked_texture.video_object() {
                textures.push(("u_MainTex", texture_handle));
            }
        }

        application
            .graphics
            .draw(0,
                  *vso,
                  *self.pipeline,
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
    priority: f32,
}

impl PartialEq for SpriteOrd {
    fn eq(&self, rhs: &SpriteOrd) -> bool {
        self.priority == rhs.priority
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
        self.priority.partial_cmp(&rhs.priority)
    }
}

fn eq(lhs: Option<&resource::TexturePtr>, rhs: Option<&resource::TexturePtr>) -> bool {
    if lhs.is_none() && rhs.is_none() {
        return true;
    }

    if lhs.is_none() || rhs.is_none() {
        return false;
    }

    let lhs = lhs.unwrap();
    let rhs = rhs.unwrap();
    let lhs: *const RwLock<resource::Texture> = &**lhs;
    let rhs: *const RwLock<resource::Texture> = &**rhs;
    lhs == rhs
}
