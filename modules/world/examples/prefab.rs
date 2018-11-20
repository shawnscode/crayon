extern crate crayon;
extern crate crayon_world;

use crayon::prelude::*;
use crayon_world::prelude::*;

struct WindowResources {
    cornell_box: PrefabHandle,
}

impl LatchProbe for WindowResources {
    fn is_set(&self) -> bool {
        crayon_world::prefab_state(self.cornell_box) != ResourceState::NotReady
    }
}

impl WindowResources {
    pub fn new() -> crayon::errors::Result<Self> {
        crayon_world::setup()?;
        Ok(WindowResources {
            cornell_box: crayon_world::create_prefab_from("res:cornell_box.obj")?,
        })
    }
}

impl Drop for WindowResources {
    fn drop(&mut self) {
        crayon_world::delete_prefab(self.cornell_box);
        crayon_world::discard();
    }
}

struct Window {
    scene: Scene<SimpleRenderer>,
    room: Entity,
}

impl Window {
    fn new(resources: &WindowResources) -> crayon::errors::Result<Self> {
        //
        let mut scene = Scene::new(SimpleRenderer::new()?);
        let room = scene.instantiate(resources.cornell_box).unwrap();

        //
        let short_box = scene.find("cornell_box.obj/shortBox").unwrap();
        let mut mtl = SimpleMaterial::default();
        mtl.diffuse = [255, 100, 100, 255].into();
        scene.add_mtl(short_box, mtl);

        let tall_box = scene.find("cornell_box.obj/tallBox").unwrap();
        let mut mtl = SimpleMaterial::default();
        mtl.diffuse = [55, 55, 255, 255].into();
        scene.add_mtl(tall_box, mtl);

        //
        let lit = scene.create("Lit");
        scene.add_lit(lit, Lit::default());
        scene.set_rotation(lit, Euler::new(Deg(45.0), Deg(0.0), Deg(0.0)));

        //
        let camera = scene.create("Main Camera");
        scene.add_camera(camera, Camera::ortho(3.2, 2.4, 0.1, 5.0));
        scene.set_position(camera, [0.0, 1.0, -1.0]);
        scene.look_at(camera, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        Ok(Window {
            room: room,
            scene: scene,
        })
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> crayon::errors::Result<()> {
        self.scene.draw();

        if let GesturePan::Move { movement, .. } = input::finger_pan() {
            let rotation = Euler::new(Deg(movement.y), Deg(-movement.x), Deg(0.0));
            self.scene.rotate(self.room, rotation);
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
    params.window.title = "CR: Prefab".into();
    params.window.size = (640, 480).into();
    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    params.input.touch_emulation = true;

    crayon::application::setup(params, || {
        let resources = WindowResources::new()?;
        Ok(Launcher::new(resources, |r| Window::new(r)))
    }).unwrap();
});
