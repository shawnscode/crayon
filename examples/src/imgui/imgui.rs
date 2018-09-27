extern crate crayon_testbed;
use crayon_testbed::prelude::*;

struct Window {
    canvas: ConsoleCanvas,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        Ok(Window {
            canvas: ConsoleCanvas::new(&ctx, Color::white())?,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let ui = self.canvas.render(ctx);
        let mut open = true;
        ui.show_test_window(&mut open);

        if !open {
            ctx.shutdown();
        }

        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let params = crayon_testbed::settings("CR: ImGui", (768, 768));
    let mut engine = Engine::new_with(&params).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
