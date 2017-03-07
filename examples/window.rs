#[macro_use]
extern crate lemon3d;

use lemon3d::graphics::*;

// Shader sources
static VS: &'static str = "#version 150\nin vec2 Position; out vec2 f_Color;\nvoid main() \
                           {\ngl_Position = vec4(Position, 0.0, 1.0);\nf_Color = Position;\n}";

static FS: &'static str = "#version 150\nin vec2 f_Color;\nout vec4 out_color;\nvoid main() \
                           {\nout_color = vec4(f_Color, 0.0, 1.0);\n}";

static VS_2: &'static str = "
#version 150
in vec2 Position;
out vec2 UV;
void main(){
    gl_Position = vec4(Position, 0.0, 1.0);
    UV = (Position+vec2(1,1))/2.0;
}";

static FS_2: &'static str = "#version 330 core
in vec2 UV;
out vec3 color;
uniform sampler2D \
                             renderedTexture;
uniform float time;
void main(){
    color = \
                             texture( renderedTexture, UV + 0.005*vec2( \
                             sin(time+1024.0*UV.x),cos(time+768.0*UV.y)) ).xyz;
}";

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}


fn main() {
    let mut rendered_texture = TextureHandle::default();
    let mut fbo = FrameBufferHandle::default();
    let mut view_fb = ViewHandle::default();
    let mut pipeline_fb = PipelineHandle::default();
    let mut vbo_fb = VertexBufferHandle::default();
    let mut view = ViewHandle::default();
    let mut pipeline = PipelineHandle::default();
    let mut vbo = VertexBufferHandle::default();
    let mut time = 0.0;

    lemon3d::Application::setup("examples/resources/configs/basic.json")
        .unwrap()
        .perform(|app| {
            let vertices: [Vertex; 3] =
                [Vertex::new([0.0, 0.5]), Vertex::new([0.5, -0.5]), Vertex::new([-0.5, -0.5])];

            let quad_vertices: [Vertex; 6] = [Vertex::new([-1.0, -1.0]),
                                              Vertex::new([1.0, -1.0]),
                                              Vertex::new([-1.0, 1.0]),
                                              Vertex::new([-1.0, 1.0]),
                                              Vertex::new([1.0, -1.0]),
                                              Vertex::new([1.0, 1.0])];

            let attributes =
                AttributeLayoutBuilder::new().with(VertexAttribute::Position, 2).finish();

            let layout = Vertex::layout();

            //
            vbo_fb = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
                                      24,
                                      Some(Vertex::as_bytes(&vertices[..])))
                .unwrap();

            let state = RenderState::default();
            rendered_texture =
                app.graphics.create_render_texture(RenderTextureFormat::RGBA8, 568, 320).unwrap();

            fbo = app.graphics
                .create_framebuffer(FrameBufferAttachment::Texture(rendered_texture),
                                    Some(color::Color::gray()),
                                    None,
                                    None)
                .unwrap();
            view_fb = app.graphics.create_view(Some(fbo)).unwrap();
            pipeline_fb = app.graphics.create_pipeline(VS, FS, &state, &attributes).unwrap();

            vbo = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
                                      48,
                                      Some(Vertex::as_bytes(&quad_vertices[..])))
                .unwrap();
            view = app.graphics.create_view(None).unwrap();
            pipeline = app.graphics.create_pipeline(VS_2, FS_2, &state, &attributes).unwrap();
        })
        .run(move |app| {
            let mut uniforms = vec![];
            let mut textures = vec![];
            app.graphics
                .draw(0,
                      view_fb,
                      pipeline_fb,
                      textures.as_slice(),
                      uniforms.as_slice(),
                      vbo_fb,
                      None,
                      Primitive::Triangles,
                      0,
                      6)
                .unwrap();

            textures.push(("renderedTexture", rendered_texture));
            uniforms.push(("time", UniformVariable::F32(time)));
            app.graphics
                .draw(0,
                      view,
                      pipeline,
                      textures.as_slice(),
                      uniforms.as_slice(),
                      vbo,
                      None,
                      Primitive::Triangles,
                      0,
                      12)
                .unwrap();

            time += 0.05;
            return true;
        })
        .perform(|_| println!("hello world."));
}