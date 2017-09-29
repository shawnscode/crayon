use core::application::Application;
use ecs::{World, Entity, ArenaMutGetter};

use math;
use resource;
use graphics;

use super::*;
use super::errors::*;

pub trait Renderable {
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn material(&self) -> Option<&resource::MaterialPtr>;
}

pub struct Renderer {
    sprite_renderer: SpriteRenderer,
    mesh_renderer: MeshRenderer,
    ambient: (math::Color, f32),
}

impl Renderer {
    pub fn new(mut app: &mut Application) -> Result<Renderer> {
        Ok(Renderer {
               sprite_renderer: SpriteRenderer::new(&mut app)?,
               mesh_renderer: MeshRenderer::new(&mut app)?,
               ambient: (math::Color::white(), 1.0f32),
           })
    }

    pub fn set_ambient_color(&mut self, color: math::Color, intensity: f32) {
        self.ambient = (color, intensity);
    }

    pub fn draw(&mut self, mut app: &mut Application, world: &World) -> Result<()> {
        // Collect all the enable camera in the world.
        let cameras = {
            let mut cameras = Vec::new();
            let (view, mut arenas) = world.view_with_2::<Transform, Camera>();
            for v in view {
                if let Ok(camera) = Renderer::parse_render_camera(&mut app.graphics,
                                                                  &mut arenas,
                                                                  v) {
                    cameras.push(camera);
                }
            }
            cameras
        };

        let env = self.parse_render_env(&world);

        // Draw from the viewport of camera.
        for v in cameras {
            self.mesh_renderer.draw(&mut app, &world, &env, &v)?;
            self.sprite_renderer.draw(&mut app, &world, &v)?;
        }

        Ok(())
    }

    fn parse_render_camera(mut video: &mut graphics::GraphicsFrontend,
                           arenas: &mut (ArenaMutGetter<Transform>, ArenaMutGetter<Camera>),
                           camera: Entity)
                           -> Result<RenderCamera> {
        let (view, project) = {
            (Camera::view_matrix(&arenas.0, camera)?, Camera::projection_matrix(&arenas.1, camera)?)
        };

        let mut c = arenas.1.get_mut(camera).unwrap();
        c.update_video_object(&mut video)?;

        Ok(RenderCamera {
               view: view,
               projection: project,
               clip: c.clip_plane(),
               vso: c.video_object().unwrap(),
           })
    }

    fn parse_render_env(&self, world: &World) -> RenderEnvironment {
        let mut env = RenderEnvironment {
            ambient: self.ambient.0,
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
        };

        let (view, arenas) = world.view_with_2::<Transform, Light>();
        for v in view {
            let light = arenas.1.get(*v).unwrap();
            let decomposed = Transform::world_decomposed(&arenas.0, v).unwrap();

            if light.is_enable() {
                match light {
                    &Light::Directional(v) => env.directional_lights.push((decomposed, v)),
                    &Light::Point(v) => env.point_lights.push((decomposed, v)),
                }
            }
        }

        return env;
    }
}

pub struct RenderEnvironment {
    pub ambient: math::Color,
    pub directional_lights: Vec<(Decomposed, DirectionalLight)>,
    pub point_lights: Vec<(Decomposed, PointLight)>,
}

#[derive(Debug)]
pub struct RenderCamera {
    pub view: math::Matrix4<f32>,
    pub projection: math::Matrix4<f32>,
    pub vso: graphics::ViewStateHandle,
    pub clip: (f32, f32),
}

impl RenderCamera {
    pub fn into_view_space(&self, position: &math::Vector3<f32>) -> math::Vector3<f32> {
        let homo = math::Vector4::new(position.x, position.y, position.z, 1.0f32);
        let p = self.view * homo;
        let p = p / p.w;
        p.truncate()
    }

    pub fn is_inside(&self, position: &math::Vector3<f32>) -> bool {
        position.z <= -self.clip.0 && position.z > -self.clip.1
    }
}