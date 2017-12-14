use crayon::prelude::*;

pub(crate) use crayon_imgui::prelude::*;
pub(crate) use crayon_imgui;

use utils;

struct Window {
    canvas: Canvas,
    renderer: Renderer,
    info: FrameInfo,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        let ctx = engine.context().read().unwrap();
        let (canvas, renderer) = crayon_imgui::new(&ctx).unwrap();

        Ok(Window {
               canvas: canvas,
               renderer: renderer,
               info: Default::default(),
           })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let ui = self.canvas.paint(&ctx);
        let info = self.info;
        ui.window(im_str!("ImGui & Crayon"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((0.0, 0.0), ImGuiCond::FirstUseEver)
            .size((224.0, 65.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("FPS: {:?}", info.fps));
                ui.text(im_str!("DrawCalls: {:?}, Vertices: {:?}",
                                info.video.drawcall,
                                info.video.vertices));

                ui.text(im_str!("CPU: {:.2?}ms, GPU: {:.2?}ms",
                                utils::to_ms(info.duration),
                                utils::to_ms(info.video.duration)));
            });

        let mut open = true;
        ui.show_test_window(&mut open);

        if !open {
            ctx.shutdown();
        }

        self.renderer.render(ui).unwrap();
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> errors::Result<()> {
        self.info = *info;
        Ok(())
    }
}

pub fn main(title: String, _: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 1024;
    settings.window.height = 768;
    settings.window.title = title;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
