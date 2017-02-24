extern crate lemon3d;

use std::slice;
use std::mem;
use lemon3d::graphics::*;
// use lemon3d::graphics::Graphics;

// Shader sources
static VS: &'static str = "#version 150\nin vec2 Position;\nvoid main() {\ngl_Position = \
                               vec4(Position, 0.0, 1.0);\n}";

static FS: &'static str = "#version 150\nout vec4 out_color;\nvoid main() {\nout_color = \
                               vec4(1.0, 1.0, 1.0, 1.0);\n}";

// Vertex data
static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

fn main() {
    let mut view = ViewHandle::default();
    let mut pipeline = PipelineHandle::default();
    let mut vbo = VertexBufferHandle::default();
    // let mut ibo;

    lemon3d::Application::setup("examples/resources/configs/basic.json")
        .unwrap()
        .perform(|app| {
            let mut attributes = vec![];
            attributes.push(VertexAttributeDesc {
                name: VertexAttribute::Position,
                format: VertexFormat::Float,
                size: 2,
                normalized: false,
            });

            let state = RenderState::default();

            view = app.graphics.create_view(Some(Color::gray()), None, None).unwrap();
            pipeline = app.graphics.create_pipeline(VS, FS, &state, attributes.as_slice()).unwrap();

            let layout = VertexLayout::build()
                .with(VertexAttribute::Position, VertexFormat::Float, 2, false)
                .finish();
            vbo = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
                                      24,
                                      Some(as_bytes(&VERTEX_DATA[..])))
                .unwrap();
        })
        .run(|app| {
            let uniforms = vec![];
            let textures = vec![];
            app.graphics
                .draw(view,
                      pipeline,
                      textures.as_slice(),
                      uniforms.as_slice(),
                      vbo,
                      None,
                      Primitive::Triangles,
                      0,
                      6)
                .unwrap();
            return true;
        })
        .perform(|_| println!("hello world."));
}

fn as_bytes<T>(values: &[T]) -> &[u8]
    where T: Copy
{
    let len = values.len() * mem::size_of::<T>();
    unsafe { slice::from_raw_parts(values.as_ptr() as *const u8, len) }
}