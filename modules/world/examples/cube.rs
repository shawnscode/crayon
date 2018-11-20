extern crate crayon;
extern crate crayon_world;

use crayon::prelude::*;
use crayon_world::prelude::*;

struct Window {
    scene: Scene<SimpleRenderer>,
    cube: Entity,
}

impl Window {
    fn new() -> crayon::errors::Result<Self> {
        crayon_world::setup()?;

        let mut scene = Scene::new(SimpleRenderer::new()?);

        //
        let cube = scene.create("Cube");
        let mesh = crayon_world::default().cube;
        let mut mtl = SimpleMaterial::default();
        mtl.diffuse_texture = Some(video::create_texture_from("res:crate.bmp")?);
        scene.add_mesh(cube, mesh);
        scene.add_mtl(cube, mtl);

        //
        let lit = scene.create("Lit");
        scene.add_lit(lit, Lit::default());

        //
        let camera = scene.create("Main Camera");
        let params = Camera::ortho(3.2, 2.4, 0.1, 5.0);
        let center = [0.0, 0.0, 0.0];
        scene.add_camera(camera, params);
        scene.set_position(camera, [0.0, 0.0, -2.0]);
        scene.look_at(camera, center, [0.0, 1.0, 0.0]);

        Ok(Window {
            scene: scene,
            cube: cube,
        })
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> crayon::errors::Result<()> {
        self.scene.draw();

        if let GesturePan::Move { movement, .. } = input::finger_pan() {
            let rotation = Euler::new(Deg(movement.y), Deg(-movement.x), Deg(0.0));
            self.scene.rotate(self.cube, rotation);
        }

        Ok(())
    }
}

main!({
    #[cfg(not(target_arch = "wasm32"))]
    let res = format!(
        "file://{}/../../examples/resources/",
        env!("CARGO_MANIFEST_DIR")
    );

    #[cfg(target_arch = "wasm32")]
    let res = format!("http://localhost:8080/examples/resources/");

    let mut params = Params::default();
    params.window.title = "CR: Cube".into();
    params.window.size = (640, 480).into();
    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    params.input.touch_emulation = true;
    crayon::application::setup(params, || Window::new()).unwrap();
});
