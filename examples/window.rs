extern crate lemon3d;

// use std::mem;

// Shader sources
// static VS_SRC: &'static str = "#version 150\nin vec2 position;\nvoid main() {\ngl_Position = \
//                                vec4(position, 0.0, 1.0);\n}";

// static FS_SRC: &'static str = "#version 150\nout vec4 out_color;\nvoid main() {\nout_color = \
//                                vec4(1.0, 1.0, 1.0, 1.0);\n}";

// // Vertex data
// static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

fn main() {
    lemon3d::Application::setup("examples/resources/configs/basic.json")
        .unwrap()
        .perform(|_| {
            // let view = lemon3d::graphics::ViewObject::new()
            //     .with_clear(Some([0.75, 0.75, 0.75, 1.0]), None, None)
            //     .with_viewport((0, 0), (128, 128));

            // let view2 = lemon3d::graphics::ViewObject::new().with_viewport((128, 128), (256, 256));

            // application.renderer.create_view(&view);
            // application.renderer.create_view(&view2);
        })
        .run(|_| {
            return true;
        })
        .perform(|_| println!("hello world."));
}
