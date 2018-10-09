use std::sync::Arc;

use crayon::errors::*;
use crayon::prelude::*;

use crayon_imgui::{self, Canvas, ImGuiCond};

fn to_ms(duration: ::std::time::Duration) -> f32 {
    duration.as_secs() as f32 * 1000.0 + duration.subsec_nanos() as f32 / 1_000_000.0
}

pub struct ConsoleCanvas {
    surface: Option<SurfaceHandle>,
    canvas: Canvas,
    info: FrameInfo,
    video: Arc<VideoSystemShared>,
}

impl Drop for ConsoleCanvas {
    fn drop(&mut self) {
        if let Some(surface) = self.surface {
            self.video.delete_surface(surface);
        }
    }
}

impl ConsoleCanvas {
    pub fn new<C>(ctx: &Context, clear_color: C) -> Result<Self>
    where
        C: Into<Option<Color<f32>>>,
    {
        let canvas = Canvas::new(ctx).unwrap();

        let surface = if let Some(color) = clear_color.into() {
            let mut params = SurfaceParams::default();
            params.set_clear(color, None, None);
            Some(ctx.video.create_surface(params)?)
        } else {
            None
        };

        Ok(ConsoleCanvas {
            surface: surface,
            canvas: canvas,
            video: ctx.video.clone(),
            info: Default::default(),
        })
    }

    pub fn update(&mut self, info: &FrameInfo) {
        self.info = *info;
    }

    pub fn render<'a>(&'a mut self, ctx: &Context) -> crayon_imgui::canvas::FrameGuard<'a> {
        let ui = self.canvas.frame(ctx, self.surface);
        let info = self.info;

        ui.window(im_str!("ImGui & Crayon"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((0.0, 0.0), ImGuiCond::FirstUseEver)
            .size((256.0, 64.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("FPS: {:?}", info.fps));
                ui.text(im_str!(
                    "DrawCalls: {:?}, Triangles: {:?}",
                    info.video.drawcall,
                    info.video.triangles
                ));

                ui.text(im_str!(
                    "CPU: {:.2?}ms, GPU: {:.2?}ms",
                    to_ms(info.duration),
                    to_ms(info.video.duration)
                ));
            });

        ui
    }
}
