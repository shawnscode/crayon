//! A simple particle program to demostrate how to use sprite component.

extern crate crayon;
extern crate rand;

use crayon::prelude::*;

struct Window {
    scene: Scene,
    cubes: Vec<Entity>,
    dir: Entity,
}

impl Window {
    fn new(mut app: &mut Application) -> errors::Result<Window> {
        let mut scene = Scene::new(&mut app)?;

        {
            // Create and bind main camera of scene.
            let c = Scene::camera(&mut scene.world_mut());
            scene.set_main_camera(c);

            {
                let dimensions = app.window.dimensions().unwrap();
                let mut camera = scene.world_mut().get_mut::<Camera>(c).unwrap();
                camera.set_aspect(dimensions.0 as f32 / dimensions.1 as f32);
                camera.set_projection(Projection::Perspective(30.0));
            }

            {
                let mut arena = scene.world_mut().arena_mut::<Transform>().unwrap();
                let mut position = Transform::world_position(&arena, c).unwrap();
                position.z = 500f32;
                Transform::set_world_position(&mut arena, c, position).unwrap();
            }
        }

        let cubes = {
            let mut cubes = Vec::new();
            for i in 0..1 {
                let cube = Window::spwan(&mut app, &mut scene)?;
                let mut transform = scene.world().get_mut::<Transform>(cube).unwrap();

                transform.set_scale(30.0);
                transform.translate(math::Vector3::unit_x() * i as f32 * 60.0);
                cubes.push(cube);
            }
            cubes
        };

        let dir = {
            let unit = math::Vector3::unit_x();
            Window::spawn_point_light(&mut app, &mut scene, Color::red(), unit * 80.0)?;
            Window::spawn_point_light(&mut app, &mut scene, Color::green(), unit * -80.0)?;

            let unit = math::Vector3::unit_z();
            let light = Window::spawn_dir_light(&mut app, &mut scene, Color::white(), unit * 80.0)?;
            let center = scene
                .world_mut()
                .build()
                .with_default::<Transform>()
                .finish();

            {
                let mut arena = scene.world().arena_mut::<Transform>().unwrap();
                Transform::set_parent(&mut arena, light, Some(center), true)?;
            }

            center
        };

        Ok(Window {
               scene: scene,
               cubes: cubes,
               dir: dir,
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

    fn spawn_color_cube(mut app: &mut Application,
                        scene: &mut Scene,
                        color: Color,
                        position: math::Vector3<f32>)
                        -> errors::Result<Entity> {
        let mesh = crayon::resource::factory::primitive::cube(&mut app.resources)?;
        let mat = crayon::resource::factory::material::color(&mut app.resources)?;

        {
            let mut mat = mat.write().unwrap();
            let color = graphics::UniformVariable::Vector3f(color.rgb());
            mat.set_uniform_variable("u_Color", color)?;
        }

        let cube = scene
            .world_mut()
            .build()
            .with_default::<Transform>()
            .with::<Mesh>(Mesh::new(mesh, Some(mat)))
            .finish();

        {
            let mut arena = scene.world_mut().arena_mut::<Transform>().unwrap();
            Transform::set_world_position(&mut arena, cube, position)?;
            Transform::set_world_scale(&mut arena, cube, 5f32)?;
        }

        Ok(cube)
    }

    fn spawn_point_light(mut app: &mut Application,
                         mut scene: &mut Scene,
                         color: Color,
                         position: math::Vector3<f32>)
                         -> errors::Result<Entity> {
        let cube = Window::spawn_color_cube(&mut app, &mut scene, color, position)?;

        let mut data = PointLight::default();
        data.color = color;
        scene.world_mut().add::<Light>(cube, Light::Point(data));

        Ok(cube)
    }

    fn spawn_dir_light(mut app: &mut Application,
                       mut scene: &mut Scene,
                       color: Color,
                       position: math::Vector3<f32>)
                       -> errors::Result<Entity> {
        let cube = Window::spawn_color_cube(&mut app, &mut scene, color, position)?;

        let mut data = DirectionalLight::default();
        data.color = color;
        scene
            .world_mut()
            .add::<Light>(cube, Light::Directional(data));

        Ok(cube)
    }
}

impl ApplicationInstance for Window {
    fn on_update(&mut self, mut app: &mut Application) -> errors::Result<()> {
        // Rotate cube.
        let rotation = Quaternion::from(math::Euler {
                                            x: math::Deg(1.0f32),
                                            y: math::Deg(1.0f32),
                                            z: math::Deg(0.0f32),
                                        });

        for cube in &self.cubes {
            self.scene
                .world()
                .get_mut::<Transform>(*cube)
                .unwrap()
                .rotate(rotation);
        }

        // Rotate directional light.
        let rotation = Quaternion::from(math::Euler {
                                            x: math::Deg(0.0f32),
                                            y: math::Deg(1.0f32),
                                            z: math::Deg(0.0f32),
                                        });

        self.scene
            .world()
            .get_mut::<Transform>(self.dir)
            .unwrap()
            .rotate(rotation);

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