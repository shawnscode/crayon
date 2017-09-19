//! A simple particle program to demostrate how to use sprite component.

extern crate crayon;
extern crate rand;

use crayon::prelude::*;

struct Window {
    scene: Scene,
    cubes: Vec<Entity>,
    light: Entity,
    far: f32,
    camera: Entity,
}

impl Window {
    fn new(mut app: &mut Application) -> errors::Result<Window> {
        let mut scene = Scene::new(&mut app)?;

        let camera = {
            // Create and bind main camera of scene.
            let c = Scene::camera(&mut scene.world_mut());
            scene.set_main_camera(c);

            {
                let dimensions = app.window.dimensions().unwrap();
                let mut camera = scene.world_mut().fetch_mut::<Camera>(c).unwrap();
                camera.set_aspect(dimensions.0 as f32 / dimensions.1 as f32);
                camera.set_projection(Projection::Perspective(30.0));
            }

            {
                let mut arena = scene.world_mut().arena::<Transform>().unwrap();
                let mut position = Transform::world_position(&arena, c).unwrap();
                position.z = 500f32;
                Transform::set_world_position(&mut arena, c, position).unwrap();
            }

            c
        };

        let cubes = {
            let mut cubes = Vec::new();
            for i in 0..1 {
                let cube = Window::spwan(&mut app, &mut scene)?;
                let mut transform = scene.world().fetch_mut::<Transform>(cube).unwrap();

                transform.set_scale(25.0);
                transform.translate(math::Vector3::unit_x() * i as f32 * 60.0);
                cubes.push(cube);
            }
            cubes
        };

        let light = {
            let parent = scene
                .world_mut()
                .build()
                .with_default::<Transform>()
                .finish();

            let light = Window::spawn_color_cube(&mut app, &mut scene)?;

            {
                let data = DirectionalLight::default();
                scene
                    .world_mut()
                    .assign::<Light>(light, Light::Directional(data));
            }

            {
                let mut arena = scene.world().arena::<Transform>().unwrap();

                {
                    let mut transform = arena.get_mut(*light).unwrap();
                    transform.set_scale(15.0);
                    transform.translate(math::Vector3::unit_z() * 80.0);
                }

                Transform::set_parent(&mut arena, light, Some(parent), true)?;
                Transform::look_at(&mut arena, light, parent, math::Vector3::unit_y())?;
            }

            parent
        };

        Ok(Window {
               scene: scene,
               cubes: cubes,
               light: light,
               far: 1000.0,
               camera: camera,
           })
    }

    fn spwan(mut app: &mut Application, scene: &mut Scene) -> errors::Result<Entity> {
        let mesh = crayon::resource::factory::primitive::cube(&mut app.resources)?;
        let mat = crayon::resource::factory::material::phong(&mut app.resources)?;

        {
            let mut mat = mat.write().unwrap();
            mat.set_uniform_variable("u_Ambient", Vector3::new(1.0, 0.5, 0.31).into())?;
            mat.set_uniform_variable("u_Diffuse", Vector3::new(1.0, 0.5, 0.31).into())?;
            mat.set_uniform_variable("u_Specular", Vector3::new(0.5, 0.5, 0.5).into())?;
            mat.set_uniform_variable("u_Shininess", 32.0f32.into())?;
        }

        let cube = scene
            .world_mut()
            .build()
            .with_default::<Transform>()
            .with::<Mesh>(Mesh::new(mesh, Some(mat)))
            .finish();

        Ok(cube)
    }

    fn spawn_color_cube(mut app: &mut Application, scene: &mut Scene) -> errors::Result<Entity> {
        let mesh = crayon::resource::factory::primitive::cube(&mut app.resources)?;
        let mat = crayon::resource::factory::material::color(&mut app.resources)?;

        {
            let mut mat = mat.write().unwrap();
            let color = graphics::UniformVariable::Vector3f(Color::gray().rgb());
            mat.set_uniform_variable("u_Color", color)?;
        }

        let cube = scene
            .world_mut()
            .build()
            .with_default::<Transform>()
            .with::<Mesh>(Mesh::new(mesh, Some(mat)))
            .finish();
        Ok(cube)
    }
}

impl ApplicationInstance for Window {
    fn on_update(&mut self, mut app: &mut Application) -> errors::Result<()> {
        // Rotate cube.
        let translation = if app.input.is_key_down(KeyboardButton::W) {
            Vector3::unit_z()
        } else if app.input.is_key_down(KeyboardButton::S) {
            Vector3::unit_z() * -1.0
        } else if app.input.is_key_down(KeyboardButton::D) {
            Vector3::unit_x()
        } else if app.input.is_key_down(KeyboardButton::A) {
            Vector3::unit_x() * -1.0
        } else if app.input.is_key_down(KeyboardButton::Q) {
            Vector3::unit_y()
        } else if app.input.is_key_down(KeyboardButton::E) {
            Vector3::unit_y() * -1.0
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        let rotation = Quaternion::from(math::Euler {
                                            x: math::Deg(0.0f32),
                                            y: math::Deg(1.0f32),
                                            z: math::Deg(0.0f32),
                                        });

        for cube in &self.cubes {
            self.scene
                .world()
                .fetch_mut::<Transform>(*cube)
                .unwrap()
                .translate(translation);

            // self.scene
            //     .world()
            //     .fetch_mut::<Transform>(*cube)
            //     .unwrap()
            //     .rotate(rotation);
        }

        self.scene
            .world()
            .fetch_mut::<Transform>(self.light)
            .unwrap()
            .rotate(rotation);

        {
            let mut camera = self.scene
                .world_mut()
                .fetch_mut::<Camera>(self.camera)
                .unwrap();
            camera.set_clip_plane(0.1, self.far);
        }

        // Run one frame of scene.
        self.scene.run_one_frame(&mut app)?;

        Ok(())
    }
}

fn main() {
    let mut settings = Settings::default();
    settings.engine.max_fps = 60;
    settings.window.width = 480;
    settings.window.height = 480;

    let manifest = "examples/compiled-resources/manifest";
    let mut app = Application::new_with(settings).unwrap();
    app.resources.load_manifest(manifest).unwrap();

    let mut window = Window::new(&mut app).unwrap();
    app.run(&mut window).unwrap();
}