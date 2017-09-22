//! A simple particle program to demostrate how to use sprite component.

extern crate crayon;
extern crate crayon_workflow;
extern crate rand;
mod utils;

use rand::{Rng, SeedableRng, XorShiftRng};
use crayon::prelude::*;

#[derive(Debug)]
struct SpriteParticle {
    lifetime: f32,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    rotation_vel: Quaternion<f32>,
    color: Color,
    size: Vector2<f32>,
    handle: Entity,
}

struct Window {
    scene: Scene,
    atlas: AtlasPtr,
    particles: Vec<SpriteParticle>,
    num: usize,
    rand: XorShiftRng,
}

impl Window {
    fn new(mut app: &mut Application, num: usize) -> errors::Result<Window> {
        let mut scene = Scene::new(&mut app)?;

        {
            // Create and bind main camera of scene.
            let c = Scene::camera(&mut scene.world_mut());
            scene.set_main_camera(c);

            {
                let dimensions = app.window.dimensions().unwrap();
                let mut camera = scene.world_mut().fetch_mut::<Camera>(c).unwrap();
                camera.set_aspect(dimensions.0 as f32 / dimensions.1 as f32);
                camera.set_projection(Projection::Ortho(dimensions.1 as f32 * 0.5));
                camera.set_clear(Some(Color::gray()), None, None);
            }

            {
                let mut arena = scene.world_mut().arena::<Transform>().unwrap();
                let mut position = Transform::world_position(&arena, c).unwrap();
                position.z = 10f32;
                Transform::set_world_position(&mut arena, c, position).unwrap();
            }
        }

        let atlas = app.resources.load("atlas.json")?;

        Ok(Window {
               scene: scene,
               atlas: atlas,
               particles: Vec::new(),
               num: num,
               rand: XorShiftRng::from_seed([0, 1, 2, 3]),
           })
    }

    /// Spawn a random sprite particle.
    fn spawn(mut app: &mut Application,
             mut world: &mut World,
             rand: &mut XorShiftRng,
             atlas: &AtlasPtr)
             -> SpriteParticle {
        let size = (rand.gen::<u32>() % 40) as f32 + 20.0;
        let spr = SpriteParticle {
            lifetime: (rand.gen::<u32>() % 5) as f32,
            velocity: Vector3::new((rand.gen::<i32>() % 5) as f32 + 1f32,
                                   (rand.gen::<i32>() % 5) as f32 + 1f32,
                                   0f32),
            rotation_vel: Quaternion::from(math::Euler {
                                               x: math::Deg((rand.gen::<i32>() % 10) as f32),
                                               y: math::Deg((rand.gen::<i32>() % 10) as f32),
                                               z: math::Deg((rand.gen::<i32>() % 10) as f32),
                                           }),
            acceleration: Vector3::new((rand.gen::<i32>() % 5) as f32 + 1f32,
                                       (rand.gen::<i32>() % 5) as f32 + 1f32,
                                       0f32),
            color: [rand.gen::<u8>(), rand.gen::<u8>(), rand.gen::<u8>(), 255].into(),
            size: Vector2::new(size, size),
            handle: Sprite::new(&mut world),
        };

        let mut sprite = world.fetch_mut::<Sprite>(spr.handle).unwrap();
        sprite.set_color(&spr.color);

        let mut rect = world.fetch_mut::<Rect>(spr.handle).unwrap();
        rect.set_size(&spr.size);
        rect.set_pivot(Vector2::new(0.5f32, 0.5f32));

        let name = format!("y{:?}.png", rand.gen::<u32>() % 10);
        let frame = atlas
            .read()
            .unwrap()
            .frame(&mut app.resources, &name)
            .expect(&format!("{:?} not found in atlas.", name));

        sprite.set_texture_rect(frame.position, frame.size);
        sprite.set_texture(Some(frame.texture));

        spr
    }
}

impl ApplicationInstance for Window {
    fn on_update(&mut self, mut app: &mut Application) -> errors::Result<()> {
        if self.particles.len() < self.num {
            let mut world = &mut self.scene.world_mut();
            self.particles
                .push(Window::spawn(&mut app, &mut world, &mut self.rand, &self.atlas));
        }

        // Update all the particles with time eplased.
        {
            let dt = app.engine.timestep_in_seconds();
            let world = &mut self.scene.world_mut();
            let (_, mut arenas) = world.view_with_2::<Transform, Sprite>();

            for v in &mut self.particles {
                v.velocity += v.acceleration * dt;

                {
                    let transform = arenas.0.get_mut(v.handle).unwrap();
                    transform.translate(v.velocity);
                    transform.rotate(v.rotation_vel);
                }

                v.lifetime -= dt;
            }
        }

        // Destroy entity if its expired.
        {
            let world = &mut self.scene.world_mut();

            let vec = self.particles
                .drain(..)
                .filter(|v| {
                            if v.lifetime > 0.0f32 {
                                return true;
                            }

                            world.free(v.handle);
                            return false;
                        })
                .collect();

            self.particles = vec;
        }

        // Run one frame of scene.
        self.scene.run_one_frame(&mut app)?;

        Ok(())
    }
}

fn main() {
    utils::compile();

    let mut settings = Settings::default();
    settings.engine.max_fps = 60;
    settings.window.width = 640;
    settings.window.height = 480;

    let manifest = "examples/compiled-resources/manifest";
    let mut app = Application::new_with(settings).unwrap();
    app.resources.load_manifest(manifest).unwrap();

    let mut window = Window::new(&mut app, 500).unwrap();
    app.run(&mut window).unwrap();
}