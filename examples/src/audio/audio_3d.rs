extern crate crayon_testbed;
use crayon_testbed::prelude::*;

use std::sync::Arc;

struct Window {
    audio: Arc<AudioSystemShared>,
    world: World<SimpleRenderer>,
    canvas: ConsoleCanvas,
    saturn: Entity,
    satellite: Entity,
    music_source: AudioSourceHandle,
}

impl Window {
    fn new(engine: &mut Engine, settings: Settings) -> Result<Self> {
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

        let satellite = world.create();
        world.renderables.add_mesh(satellite, cube);
        world.scene.set_parent(satellite, saturn, false).unwrap();
        world.scene.set_local_position(satellite, [3.0, 0.0, 0.0]);
        world.scene.set_local_scale(satellite, 0.1);

        //
        let center = [0.0, 0.0, 0.0];
        let camera = world.create();
        let params = Camera::perspective(math::Deg(50.0), 1.33, 0.1, 50.0);
        world.renderables.add_camera(camera, params);
        world.scene.set_position(camera, [0.0, 5.0, -5.0]);
        world.scene.look_at(camera, center, [0.0, 1.0, 0.0]);

        //
        let audio = (if settings.headless {
            AudioSystem::headless(ctx.res.clone())
        } else {
            AudioSystem::new(ctx.res.clone())
        }?).shared();

        let music = audio.create_clip_from("res:music.mp3")?;
        let mut source = AudioSource::from(music);
        source.loops = AudioSourceWrap::Infinite;
        source.spatial = AudioSourceSpatial::new(0.5, 1.5).into();

        let music_source = audio.play(source).unwrap();

        Ok(Window {
            audio: audio,
            world: world,
            canvas: ConsoleCanvas::new(&ctx, None)?,
            saturn: saturn,
            satellite: satellite,
            music_source: music_source,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let position = self.world.scene.position(self.satellite).unwrap();

        self.audio
            .set_position(self.music_source, (position.x, 0.0, 0.0));

        let rotation = math::Euler::new(
            math::Deg(0.0),
            math::Deg(ctx.time.frame_delta().subsec_millis() as f32 / 25.0),
            math::Deg(0.0),
        );

        self.world.scene.rotate(self.saturn, rotation);

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

    let window = Window::new(&mut engine, params).unwrap();
    engine.run(window).unwrap();
}
