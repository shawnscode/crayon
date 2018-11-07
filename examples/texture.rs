#[macro_use]
extern crate crayon;

use crayon::errors::*;
use crayon::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    surface: SurfaceHandle,
    shader: ShaderHandle,
    mesh: MeshHandle,
    texture: TextureHandle,
    vcmds: CommandBuffer,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();

        let verts: [Vertex; 4] = [
            Vertex::new([-1.0, -1.0]),
            Vertex::new([1.0, -1.0]),
            Vertex::new([1.0, 1.0]),
            Vertex::new([-1.0, 1.0]),
        ];
        let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

        // Create vertex buffer object.
        let mut params = MeshParams::default();
        params.num_verts = 4;
        params.num_idxes = 6;
        params.layout = Vertex::layout();

        let data = MeshData {
            vptr: Vertex::encode(&verts[..]).into(),
            iptr: IndexFormat::encode(&idxes).into(),
        };

        let mesh = ctx.video.create_mesh(params, Some(data))?;

        // Create the view state.
        let setup = SurfaceParams::default();
        let surface = ctx.video.create_surface(setup)?;

        // Create shader state.
        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 2)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("renderedTexture", UniformVariableType::Texture)
            .finish();

        let mut params = ShaderParams::default();
        params.attributes = attributes;
        params.uniforms = uniforms;
        let vs = include_str!("shaders/texture.vs").to_owned();
        let fs = include_str!("shaders/texture.fs").to_owned();
        let shader = ctx.video.create_shader(params, vs, fs)?;
        let texture = ctx.video.create_texture_from("res:crate.bmp")?;

        Ok(Window {
            surface: surface,
            shader: shader,
            mesh: mesh,
            texture: texture,
            vcmds: CommandBuffer::new(),
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let mut dc = Draw::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        self.vcmds.draw(dc);
        self.vcmds.submit(&ctx.video, self.surface)?;
        Ok(())
    }

    fn on_exit(&mut self, ctx: &Context) -> Result<()> {
        ctx.video.delete_mesh(self.mesh);
        ctx.video.delete_shader(self.shader);
        ctx.video.delete_surface(self.surface);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut params = Settings::default();
    params.window.title = "CR: Texture".into();
    params.window.size = (464, 434).into();

    let mut engine = Engine::new_with(&params).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}

#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_main() {
    crayon::application::prelude::sys::init();

    let mut params = Settings::default();
    params.window.title = "CR: Texture".into();
    params.window.size = (464, 434).into();

    let mut engine = Engine::new_with(&params).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run_wasm(window).unwrap();
}

// pub fn find_res_dir() -> crayon::res::vfs::Directory {
//     use std::path::Path;

//     let root = Path::new(env!("CARGO_MANIFEST_DIR"));
//     let search_dirs = [root.join("examples").join("resources")];

//     for v in &search_dirs {
//         if v.is_dir() && v.join(crayon::res::vfs::manifest::NAME).exists() {
//             return crayon::res::vfs::Directory::new(v).unwrap();
//         }
//     }

//     panic!("Could not found compiled resources.");
// }
