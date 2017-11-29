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
    fps: Entity,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        engine
            .resource
            .mount("std",
                   resource::filesystem::DirectoryFS::new("examples/resources")?)?;

        let ctx = engine.context().read().unwrap();
        let mut canvas = CanvasSystem::new(&ctx, (640.0, 480.0), 2.0).unwrap();

        Self::create_text(&mut canvas,
                          [320.0, 240.0],
                          [0.5, 0.5],
                          "Hello, World!",
                          64,
                          Color::blue());

        let fps = Self::create_text(&mut canvas,
                                    [0.0, 480.0],
                                    [0.0, 1.0],
                                    "FPS: 30",
                                    16,
                                    Color::black());

        Ok(Window {
               canvas: canvas,
               fps: fps,
           })
    }

    fn create_text(sys: &mut CanvasSystem,
                   position: [f32; 2],
                   pivot: [f32; 2],
                   n: &str,
                   size: u32,
                   color: Color)
                   -> Entity {
        let node = sys.create();

        let mut text = Text::default();
        text.text = n.to_owned();
        text.color = color;
        text.size = size;

        sys.set_element(node, Element::Text(text));

        let mut layout = Layout::default();
        layout.set_position(position);
        layout.set_pivot(pivot);

        sys.set_layout(node, layout);
        node
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        self.canvas.advance().unwrap();
        self.canvas.perform_layout(ctx).unwrap();
        self.canvas.draw(ctx).unwrap();
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> errors::Result<()> {
        let mut text = Text::default();
        text.text = format!("FPS: {:?}\n{:#?}", info.fps, info.video);
        text.color = Color::black();
        text.size = 16;

        self.canvas.set_element(self.fps, Element::Text(text));
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