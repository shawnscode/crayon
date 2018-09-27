extern crate crayon_testbed;
use crayon_testbed::prelude::*;

struct Window {
    world: World<SimpleRenderer>,
    canvas: ConsoleCanvas,
    cube: Entity,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        //
        let world_resources = (WorldResources::new(engine)?).shared();

        let ctx = engine.context();
        let pipeline = SimpleRenderer::new(&ctx, world_resources.clone())?;

        let mut world = World::new(world_resources.clone(), pipeline);

        //
        let cube = world.create();

        let mesh = world_resources.meshes.cube;
        world.renderables.add_mesh(cube, mesh);

        // Lets give cube a crate texture.
        let mut m = SimpleMaterial::default();
        m.diffuse_texture = Some(ctx.video.create_texture_from("res:crate.bmp")?);
        world.renderer.add(cube, m);

        //
        let lit = world.create();
        world.renderables.add_lit(lit, Lit::default());

        //
        let camera = world.create();
        let params = Camera::ortho(3.2, 2.4, 0.1, 5.0);
        let center = [0.0, 0.0, 0.0];
        world.renderables.add_camera(camera, params);
        world.scene.set_position(camera, [0.0, 0.0, -2.0]);
        world.scene.look_at(camera, center, [0.0, 1.0, 0.0]);

        Ok(Window {
            world: world,
            canvas: ConsoleCanvas::new(&ctx, None)?,
            cube: cube,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        self.world.advance();

        if let GesturePan::Move { movement, .. } = ctx.input.finger_pan() {
            let rotation = Euler::new(Deg(movement.y), Deg(-movement.x), Deg(0.0));

            self.world.scene.rotate(self.cube, rotation);
        }

        self.canvas.render(ctx);
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let params = crayon_testbed::settings("CR: Cube", (640, 480));
    let mut engine = Engine::new_with(&params).unwrap();
    let res = crayon_testbed::find_res_dir();
    engine.res.mount("res", res).unwrap();
    engine.input.set_touch_emulation(true);

    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
