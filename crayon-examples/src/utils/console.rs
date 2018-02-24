use std::sync::Arc;

use crayon::prelude::*;
use crayon_imgui;
use crayon_imgui::prelude::*;

use utils;
use errors::*;

pub struct ConsoleCanvas {
    canvas: Canvas,
    info: FrameInfo,
    surface: SurfaceHandle,
    video: Arc<GraphicsSystemShared>,
}

impl Drop for ConsoleCanvas {
    fn drop(&mut self) {
        self.video.delete_surface(self.surface);
    }
}

impl ConsoleCanvas {
    pub fn new(order: u64, ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<GraphicsSystem>().clone();
        let canvas = Canvas::new(ctx).unwrap();

        let mut setup = graphics::SurfaceSetup::default();
        setup.set_clear(None, None, None);
        setup.set_sequence(true);
        setup.set_order(order);
        let surface = video.create_surface(setup)?;

        Ok(ConsoleCanvas {
            surface: surface,
            canvas: canvas,
            info: Default::default(),
            video: video,
        })
    }

    pub fn update(&mut self, info: &FrameInfo) {
        self.info = *info;
    }

    pub fn render<'a>(&'a mut self, ctx: &Context) -> crayon_imgui::canvas::FrameGuard<'a> {
        let ui = self.canvas.frame(self.surface, ctx);
        let info = self.info;
        ui.window(im_str!("ImGui & Crayon"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((0.0, 0.0), ImGuiCond::FirstUseEver)
            .size((250.0, 65.0), ImGuiCond::FirstUseEver)
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

        ui
    }
}
