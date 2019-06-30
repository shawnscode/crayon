extern crate crayon;
extern crate crayon_bytes;

use crayon::prelude::*;
use crayon_bytes::prelude::*;


#[derive(Debug, Clone, Copy)]
struct WindowResources {
    b: BytesHandle,
}

impl WindowResources {
    pub fn new() -> CrResult<Self> {
        crayon_bytes::setup()?;
        Ok(WindowResources {
            b: crayon_bytes::create_bytes_from("res:crate.bmp")?,
        })
    }
}
impl LatchProbe for WindowResources {
    fn is_set(&self) -> bool {
        crayon_bytes::state(self.b) != ResourceState::NotReady
    }
}

struct Window {
    resources: WindowResources,
}

impl Drop for Window {
    fn drop(&mut self) {
        crayon_bytes::discard();
    }
}

impl Window {
    fn new(resources: &WindowResources) -> CrResult<Self> {

        Ok(Window {
            resources: *resources,
        })
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> CrResult<()> {

        Ok(())
    }
}

main!({
    #[cfg(not(target_arch = "wasm32"))]
    let res = format!(
        "file://{}/../../examples/resources/",
        env!("CARGO_MANIFEST_DIR")
    );
    #[cfg(not(target_arch = "wasm32"))]
    let size = (640, 128);

    #[cfg(target_arch = "wasm32")]
    let res = format!("http://localhost:8080/examples/resources/");
    #[cfg(target_arch = "wasm32")]
    let size = (256, 256);

    let mut params = Params::default();
    params.window.size = size.into();
    params.window.title =
        "CR: Audio (Key[K]: Play Sound Effect; Key[1]: Increase Volume; Key[2] Decrease Volume)"
            .into();

    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    crayon::application::setup(params, || {
        let resources = WindowResources::new()?;
        Ok(Launcher::new(resources, |r| Window::new(r)))
    }).unwrap();
});
