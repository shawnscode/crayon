extern crate crayon;
extern crate crayon_audio;

use crayon::prelude::*;
use crayon_audio::prelude::*;

#[derive(Debug, Clone, Copy)]
struct WindowResources {
    sfx: AudioClipHandle,
    music: AudioClipHandle,
}

impl LatchProbe for WindowResources {
    fn is_set(&self) -> bool {
        crayon_audio::clip_state(self.sfx) != ResourceState::NotReady
            && crayon_audio::clip_state(self.music) != ResourceState::NotReady
    }
}

impl WindowResources {
    pub fn new() -> CrResult<Self> {
        crayon_audio::setup()?;
        Ok(WindowResources {
            music: crayon_audio::create_clip_from("res:music.mp3")?,
            sfx: crayon_audio::create_clip_from("res:sfx.ogg")?,
        })
    }
}

struct Window {
    resources: WindowResources,
    music_source: AudioSourceHandle,
    music_volume: f32,
    // music_pitch: f32,
}

impl Drop for Window {
    fn drop(&mut self) {
        crayon_audio::delete_clip(self.resources.music);
        crayon_audio::delete_clip(self.resources.sfx);
        crayon_audio::discard();
    }
}

impl Window {
    fn new(resources: &WindowResources) -> CrResult<Self> {
        let mut params = AudioSource::from(resources.music);
        params.loops = AudioSourceWrap::Infinite;

        Ok(Window {
            resources: *resources,
            music_source: crayon_audio::play(params)?,
            music_volume: 1.0,
            // music_pitch: 1.0,
        })
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> CrResult<()> {
        if input::is_key_down(Key::K) {
            crayon_audio::play(self.resources.sfx)?;
        }

        if input::is_key_down(Key::Key1) {
            self.music_volume += 0.1;
            crayon_audio::set_volume(self.music_source, self.music_volume);
        }

        if input::is_key_down(Key::Key2) {
            self.music_volume -= 0.1;
            crayon_audio::set_volume(self.music_source, self.music_volume);
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
