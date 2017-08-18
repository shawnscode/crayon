extern crate crayon;
extern crate cgmath;
extern crate rand;

use crayon::scene::camera::{Camera, Projection};
use crayon::scene::scene2d::Scene2d;
use cgmath as math;
use rand::{Rng, SeedableRng, XorShiftRng};

use crayon::graphics::Color;
use crayon::scene::{Sprite, Transform, Rect};
use crayon::resource;

#[derive(Debug)]
struct SpriteParticle {
    lifetime: f32,
    velocity: math::Vector3<f32>,
    acceleration: math::Vector3<f32>,
    color: Color,
    size: math::Vector2<f32>,
    handle: crayon::ecs::Entity,
}

fn main() {
    let mut scene: Option<Scene2d> = None;
    let mut atlas: Option<resource::AtlasItem> = None;
    let mut particles = vec![];
    let mut cal = XorShiftRng::from_seed([0, 1, 2, 3]);

    crayon::Application::new()
        .unwrap()
        .perform(|mut app| {
            scene = {
                let mut v = Scene2d::new(&mut app).unwrap();
                let c = Scene2d::camera(&mut v.world_mut());
                v.set_main_camera(c);

                {
                    // Create and bind main camera of scene2d.
                    let dimensions = app.window.dimensions().unwrap();
                    let mut camera = v.world_mut().fetch_mut::<Camera>(c).unwrap();
                    camera.set_aspect(dimensions.0 as f32 / dimensions.1 as f32);
                    camera.set_projection(Projection::Ortho(dimensions.1 as f32 * 0.5));
                }

                for _ in 0..200 {
                    particles.push(None);
                }

                app.resources
                    .load_manifest("crayon-runtime/examples/resources/compiled/manifest")
                    .unwrap();

                atlas = Some(app.resources.load_atlas("atlas.json").unwrap());
                Some(v)
            };
        })
        .run(move |mut app| {
            if let Some(ref mut v) = scene {
                {
                    let mut world = &mut v.world_mut();
                    // Creates sprites randomly.
                    for i in &mut particles {
                        if i.is_none() {
                            let spr = {
                                SpriteParticle {
                                    lifetime: (cal.gen::<u32>() % 5) as f32,
                                    velocity: math::Vector3::new((cal.gen::<i32>() % 10) as f32,
                                                                 (cal.gen::<i32>() % 10) as f32,
                                                                 0.0),
                                    acceleration: math::Vector3::new((cal.gen::<i32>() % 10) as
                                                                     f32,
                                                                     (cal.gen::<i32>() % 10) as
                                                                     f32,
                                                                     0.0),
                                    color: [cal.gen::<u8>(), cal.gen::<u8>(), cal.gen::<u8>(), 255]
                                        .into(),
                                    size: math::Vector2::new((cal.gen::<u32>() % 20) as f32 + 10.0,
                                                             (cal.gen::<u32>() % 20) as f32 + 10.0),
                                    handle: Scene2d::sprite(&mut world),
                                }
                            };

                            {
                                let mut sprite = world.fetch_mut::<Sprite>(spr.handle).unwrap();
                                sprite.set_additive_color(&spr.color);

                                let mut rect = world.fetch_mut::<Rect>(spr.handle).unwrap();
                                rect.set_size(&spr.size);

                                if let Some(ref atlas) = atlas {
                                    let name = format!("y{:?}.png", cal.gen::<u32>() % 10);
                                    let frame = atlas
                                        .read()
                                        .unwrap()
                                        .frame(&mut app.resources, &name)
                                        .unwrap();
                                    sprite.set_texture(Some(frame.texture));
                                }
                            }

                            *i = Some(spr);
                            break;
                        }
                    }
                }

                let mut removes = vec![];

                {
                    let dt = app.engine.timestep_in_seconds();
                    let world = &mut v.world_mut();
                    let (_, mut arenas) = world.view_with_2::<Transform, Sprite>();
                    for (i, w) in particles.iter_mut().enumerate() {
                        if let Some(ref mut particle) = *w {
                            particle.velocity += particle.acceleration * dt;
                            arenas
                                .0
                                .get_mut(particle.handle)
                                .unwrap()
                                .translate(particle.velocity);

                            particle.lifetime -= dt;
                            if particle.lifetime < 0.0 {
                                removes.push(i);
                            }
                        }
                    }
                }

                {
                    let mut world = &mut v.world_mut();
                    for i in removes {
                        if let Some(ref v) = particles[i] {
                            world.free(v.handle);
                        }
                        particles[i] = None;
                    }
                }

                v.run_one_frame(&mut app).unwrap();
            }
            return true;
        });
}