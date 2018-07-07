use crayon::prelude::*;
use crayon::video::assets::prelude::*;
use crayon_imgui::prelude::*;

use errors::*;
use utils;

struct Window {
    canvas: Canvas,
    surface: SurfaceHandle,
    info: FrameInfo,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        let canvas = Canvas::new(ctx).unwrap();

        let mut setup = SurfaceSetup::default();
        setup.set_clear(math::Color::white(), None, None);
        setup.set_sequence(true);
        let surface = ctx.video.create_surface(setup)?;

        Ok(Window {
            canvas: canvas,
            surface: surface,
            info: Default::default(),
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let ui = self.canvas.frame(self.surface, ctx);
        let info = self.info;
        ui.window(im_str!("ImGui & Crayon"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((0.0, 0.0), ImGuiCond::FirstUseEver)
            .size((224.0, 65.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("FPS: {:?}", info.fps));
                ui.text(im_str!(
                    "DrawCalls: {:?}, Triangles: {:?}",
                    info.video.drawcall,
                    info.video.triangles
                ));

                ui.text(im_str!(
                    "CPU: {:.2?}ms, GPU: {:.2?}ms",
                    utils::to_ms(info.duration),
                    utils::to_ms(info.video.duration)
                ));
            });

        let mut open = true;
        ui.show_test_window(&mut open);

        if !open {
            ctx.shutdown();
        }

        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.info = *info;
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.width = 1024;
    settings.window.height = 768;

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
