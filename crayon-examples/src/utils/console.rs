use crayon::prelude::*;

use crayon_imgui;
use crayon_imgui::prelude::*;

use errors::*;
use utils;

pub struct ConsoleCanvas {
    canvas: Canvas,
    info: FrameInfo,
}

impl ConsoleCanvas {
    pub fn new(ctx: &Context) -> Result<Self> {
        let canvas = Canvas::new(ctx).unwrap();

        Ok(ConsoleCanvas {
            canvas: canvas,
            info: Default::default(),
        })
    }

    pub fn update(&mut self, info: &FrameInfo) {
        self.info = *info;
    }

    pub fn render<'a>(&'a mut self, ctx: &Context) -> crayon_imgui::canvas::FrameGuard<'a> {
        let ui = self.canvas.frame(ctx, None);
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
