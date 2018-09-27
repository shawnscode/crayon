extern crate crayon_testbed;
use crayon_testbed::prelude::*;

struct Window {
    world: World<SimpleRenderer>,
    canvas: ConsoleCanvas,

    sun: Entity,
    saturn: Entity,
    satellites: Vec<(Entity, Quaternion<f32>)>,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        use std::f32::consts::FRAC_1_PI;

        //
        let world_resources = (WorldResources::new(engine)?).shared();

        let ctx = engine.context();
        let pipeline = SimpleRenderer::new(&ctx, world_resources.clone())?;
        let mut world = World::new(world_resources.clone(), pipeline);

        //
        let cube = world_resources.meshes.cube;
        let sphere = world_resources.meshes.sphere;

        let saturn = world.create();
        world.renderables.add_mesh(saturn, sphere);
        let mut m = SimpleMaterial::default();
        m.diffuse_texture = Some(ctx.video.create_texture_from("res:crate.bmp")?);
        world.renderer.add(saturn, m);

        let mut satellites = Vec::new();
        for i in 0..250 {
            let center = world.create();
            world.scene.set_parent(center, saturn, false).unwrap();

            let rotation = Euler::new(Deg(0.0), Deg(rand::random::<f32>()), Deg(0.0));

            satellites.push((center, rotation.into()));

            let satellite = world.create();
            world.renderables.add_mesh(satellite, cube);
            world.scene.set_parent(satellite, center, false).unwrap();

            let distance = (rand::random::<f32>() * 1.0 - 0.5) + 3.0;
            let v = 0.5 * (i as f32) * FRAC_1_PI;
            let position = [
                v.sin() * distance,
                rand::random::<f32>(),
                v.cos() * distance,
            ];

            let rotation = Euler::new(
                Deg(rand::random::<f32>() * 360.0),
                Deg(rand::random::<f32>() * 360.0),
                Deg(rand::random::<f32>() * 360.0),
            );

            world.scene.set_local_rotation(satellite, rotation);
            world.scene.set_local_position(satellite, position);
            world.scene.set_local_scale(satellite, 0.1);

            m.diffuse_texture = None;
            m.diffuse = Color::gray();
            world.renderer.add(satellite, m);
        }

        let center = [0.0, 0.0, 0.0];

        //
        let sun = world.create();
        let planet = world.create();
        let mut lit = Lit::default();
        lit.color = [255, 0, 0, 255].into();
        lit.intensity = 1.0;
        world.renderables.add_lit(planet, lit);
        world.scene.set_local_position(planet, [0.0, 4.0, 4.0]);
        world.scene.look_at(planet, center, [0.0, 1.0, 0.0]);
        world.scene.set_parent(planet, sun, false).unwrap();

        //
        let camera = world.create();
        let params = Camera::perspective(Deg(50.0), 1.33, 0.1, 50.0);
        world.renderables.add_camera(camera, params);
        world.scene.set_position(camera, [0.0, 5.0, -5.0]);
        world.scene.look_at(camera, center, [0.0, 1.0, 0.0]);

        Ok(Window {
            world: world,
            canvas: ConsoleCanvas::new(&ctx, None)?,
            sun: sun,
            saturn: saturn,
            satellites: satellites,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let rotation = Euler::new(
            Deg(0.0),
            Deg(ctx.time.frame_delta().subsec_millis() as f32 / 100.0),
            Deg(0.0),
        );

        self.world.scene.rotate(self.saturn, rotation);

        let rotation = Euler::new(
            Deg(0.0),
            Deg(ctx.time.frame_delta().subsec_millis() as f32 / 50.0),
            Deg(0.0),
        );

        self.world.scene.rotate(self.sun, rotation);

        for &(v, r) in &self.satellites {
            self.world.scene.rotate(v, r);
        }

        self.world.advance();
        self.canvas.render(ctx);
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let params = crayon_testbed::settings("CR: Saturn", (640, 480));
    let mut engine = Engine::new_with(&params).unwrap();
    let res = crayon_testbed::find_res_dir();
    engine.res.mount("res", res).unwrap();
    engine.input.set_touch_emulation(true);

    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
