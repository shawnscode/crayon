use core::Application;
use ecs;
use graphics;
use math;
use math::EuclideanSpace;
use resource;

use super::errors::*;
use super::transform::Transform;
use super::rect::Rect;
use super::sprite::Sprite;
use super::camera::Camera;

impl_vertex! {
    Vertex {
        position => [Position; Float; 2; false],
        color => [Color0; UByte; 4; true],
        texcoord => [Texcoord0; UByte; 2; true],
    }
}

const MAX_BATCH_VERTICES: usize = 1024;

pub struct Scene2d {
    view: graphics::ViewHandle,
    pso: graphics::PipelineHandle,
    vertices: Vec<Vertex>,
    world: ecs::World,
    camera: Option<ecs::Entity>,
}

impl Scene2d {
    pub fn new(application: &mut Application) -> Result<Self> {
        let attributes = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .finish();

        let view = application.graphics.create_view(None)?;
        let state = graphics::RenderState::default();
        let pso = application.graphics
            .create_pipeline(include_str!("../../resources/shaders/scene2d.vs"),
                             include_str!("../../resources/shaders/scene2d.fs"),
                             &state,
                             &attributes)?;

        let mut world = ecs::World::new();
        world.register::<Transform>();
        world.register::<Rect>();
        world.register::<Sprite>();
        world.register::<Camera>();

        Ok(Scene2d {
            world: world,
            view: view,
            pso: pso,
            camera: None,
            vertices: Vec::with_capacity(MAX_BATCH_VERTICES),
        })
    }

    pub fn world(&self) -> &ecs::World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut ecs::World {
        &mut self.world
    }

    pub fn main_camera(&self) -> Option<ecs::Entity> {
        self.camera
    }

    pub fn set_main_camera(&mut self, camera: ecs::Entity) {
        self.camera = Some(camera)
    }

    pub fn run_one_frame(&mut self, mut application: &mut Application) -> Result<()> {
        let (view_mat, proj_mat) = if let Some(id) = self.camera {
            let view_mat = {
                let arena = self.world.arena::<Transform>().unwrap();
                let dir = math::Vector3::new(0.0, 0.0, 1.0);
                let forward = Transform::transform_point(&arena, id, dir)?;
                let center = Transform::world_position(&arena, id)?;
                let up = Transform::up(&arena, id)?;
                math::Matrix4::<f32>::look_at(math::Point3::from_vec(forward),
                                              math::Point3::from_vec(center),
                                              up)
            };

            let camera = self.world.fetch::<Camera>(id).ok_or(ErrorKind::CanNotDrawWithoutCamera)?;
            let proj_mat = camera.projection_matrix();
            (view_mat, proj_mat)
        } else {
            bail!(ErrorKind::CanNotDrawWithoutCamera);
        };

        let mut main_texture = None;
        let (view, arenas) = self.world.view_with_3::<Transform, Rect, Sprite>();
        for v in view {
            let coners = Rect::world_corners(&arenas.0, &arenas.1, v).unwrap();
            let sprite = arenas.2.get(*v).unwrap();
            let color = sprite.color().into();

            self.vertices.push(Vertex::new(coners[0].into(), color, [0, 0]));
            self.vertices.push(Vertex::new(coners[3].into(), color, [0, 255]));
            self.vertices.push(Vertex::new(coners[2].into(), color, [255, 255]));
            self.vertices.push(Vertex::new(coners[0].into(), color, [0, 0]));
            self.vertices.push(Vertex::new(coners[2].into(), color, [255, 255]));
            self.vertices.push(Vertex::new(coners[1].into(), color, [255, 0]));

            let texture = sprite.texture();
            if main_texture != texture || self.vertices.len() >= MAX_BATCH_VERTICES {
                main_texture = texture;
                self.consume_vertices(&mut application, &view_mat, &proj_mat, texture)?;
                self.vertices.clear();
            }
        }

        self.consume_vertices(&mut application, &view_mat, &proj_mat, main_texture)?;
        self.vertices.clear();
        Ok(())
    }

    fn consume_vertices(&self,
                        mut application: &mut Application,
                        view_mat: &math::Matrix4<f32>,
                        proj_mat: &math::Matrix4<f32>,
                        texture: Option<resource::ResourceHandle>)
                        -> Result<()> {

        let uniforms = [("u_View", graphics::UniformVariable::Matrix4f(*view_mat.as_ref(), true)),
                        ("u_Proj", graphics::UniformVariable::Matrix4f(*proj_mat.as_ref(), true))];

        let layout = Vertex::layout();
        let vbo = application.graphics
            .create_vertex_buffer(&layout,
                                  graphics::ResourceHint::Dynamic,
                                  (self.vertices.len() * layout.stride() as usize) as u32,
                                  Some(Vertex::as_bytes(self.vertices.as_slice())))?;

        if let Some(texture) =
               texture.and_then(|v| application.resource.get::<resource::Texture>(v))
            .and_then(|v| v.video_object()) {
            application.graphics
                .draw(0,
                      self.view,
                      self.pso,
                      &[("u_MainTex", texture)],
                      &uniforms,
                      vbo,
                      None,
                      graphics::Primitive::Triangles,
                      0,
                      self.vertices.len() as u32)?;
        } else {
            application.graphics
                .draw(0,
                      self.view,
                      self.pso,
                      &[],
                      &uniforms,
                      vbo,
                      None,
                      graphics::Primitive::Triangles,
                      0,
                      self.vertices.len() as u32)?;

        }

        application.graphics.delete_vertex_buffer(vbo)?;
        Ok(())
    }
}

impl Scene2d {
    /// Create a empty sprite.
    pub fn sprite(world: &mut ecs::World) -> ecs::Entity {
        world.build()
            .with_default::<Transform>()
            .with_default::<Rect>()
            .with_default::<Sprite>()
            .finish()
    }

    pub fn camera(world: &mut ecs::World) -> ecs::Entity {
        world.build()
            .with_default::<Transform>()
            .with_default::<Camera>()
            .finish()
    }
}