#[macro_use]
extern crate crayon;
extern crate env_logger;

use crayon::errors::*;
use crayon::prelude::*;

impl_vertex! {
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
    fn build() -> Result<Self> {
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

        let mesh = video::create_mesh(params, Some(data))?;

        // Create the view state.
        let setup = SurfaceParams::default();
        let surface = video::create_surface(setup)?;

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
        let shader = video::create_shader(params, vs, fs)?;
        let texture = video::create_texture_from("res:crate.bmp")?;

        Ok(Window {
            surface,
            shader,
            mesh,
            texture,
            vcmds: CommandBuffer::new(),
        })
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        video::delete_mesh(self.mesh);
        video::delete_shader(self.shader);
        video::delete_surface(self.surface);
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> Result<()> {
        let mut dc = Draw::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        self.vcmds.draw(dc);
        self.vcmds.submit(self.surface)?;
        Ok(())
    }
}

main!({
    #[cfg(not(target_arch = "wasm32"))]
    let res = format!("file://{}/examples/resources/", env!("CARGO_MANIFEST_DIR"));
    #[cfg(target_arch = "wasm32")]
    let res = format!("http://localhost:8080/examples/resources/");

    let mut params = Params::default();
    params.window.title = "CR: Texture".into();
    params.window.size = (464, 434).into();
    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    crayon::application::setup(params, Window::build).unwrap();
});
