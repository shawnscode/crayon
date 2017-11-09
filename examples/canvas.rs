#[macro_use]
extern crate crayon;
extern crate crayon_canvas;

use crayon::prelude::*;
use crayon_canvas::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    canvas: CanvasSystem,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        engine
            .resource
            .mount("std",
                   resource::filesystem::DirectoryFS::new("examples/resources")?)?;

        let ctx = engine.context().read().unwrap();
        let mut canvas = CanvasSystem::new(&ctx).unwrap();
        canvas.create_text();

        Ok(Window { canvas: canvas })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        // self.canvas.perform_layout(ctx).unwrap();
        self.canvas.draw(ctx).unwrap();
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, _: &FrameInfo) -> errors::Result<()> {
        // println!("\nFRAME\n-----------------------\n{:#?}", info);
        Ok(())
    }
}

fn main() {
    let mut settings = Settings::default();
    settings.window.width = 640;
    settings.window.height = 480;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}