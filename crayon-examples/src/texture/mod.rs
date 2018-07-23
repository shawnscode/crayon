use crayon::prelude::*;
use crayon::video::assets::prelude::*;

use errors::*;
use utils::*;

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
    video: VideoSystemGuard,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let assets = format!("{0}/assets", env!("CARGO_MANIFEST_DIR"));
        // engine.resource.mount("std", DirectoryFS::new(assets)?)?;

        let ctx = engine.context();
        let mut video = VideoSystemGuard::new(ctx.video.clone());

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
        let verts = Some(Vertex::encode(&verts[..]));
        let idxes = Some(IndexFormat::encode(&idxes));
        let mesh = video.create_mesh(params, verts, idxes)?;

        // Create the view state.
        let setup = SurfaceParams::default();
        let surface = video.create_surface(setup)?;

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
        let vs = include_str!("../../assets/texture.vs").to_owned();
        let fs = include_str!("../../assets/texture.fs").to_owned();
        let shader = video.create_shader(params, vs, fs)?;

        // let mut ara = TextureParams::default();
        // setup.location = Location::shared("/std/texture.png");
        // let texture = video.create_texture_from_file::<TextureParser>(setup)?;

        let texture = ctx.res.load(format!("{0}/texture.png", assets).as_ref())?;

        Ok(Window {
            surface: surface,
            shader: shader,
            mesh: mesh,
            texture: texture,
            video: video,
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, _: &Context) -> Result<()> {
        let mut dc = DrawCall::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        self.video.draw(self.surface, dc);
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.size = math::Vector2::new(232, 217);

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
