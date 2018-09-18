#[macro_use]
extern crate crayon_testbed;
use crayon_testbed::prelude::*;
use crayon_testbed::ImGuiCond;

use std::sync::Arc;

struct Window {
    canvas: ConsoleCanvas,
    audio: Arc<AudioSystemShared>,
    sfx: AudioClipHandle,
    music: AudioClipHandle,
    music_source: Option<AudioSourceHandle>,
    music_volume: f32,
    music_pitch: f32,
}

impl Window {
    fn new(engine: &mut Engine, settings: Settings) -> Result<Self> {
        let ctx = engine.context();

        let audio = (if settings.headless {
            AudioSystem::headless(ctx.res.clone())
        } else {
            AudioSystem::new(ctx.res.clone())
        }?).shared();

        let sfx = audio.create_clip_from("res:sfx.ogg")?;
        let music = audio.create_clip_from("res:music.mp3")?;

        Ok(Window {
            canvas: ConsoleCanvas::new(&ctx, math::Color::white())?,
            audio: audio,
            sfx: sfx,
            music: music,
            music_source: None,
            music_volume: 1.0,
            music_pitch: 1.0,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let ui = self.canvas.render(&ctx);
        let mut play_sfx = false;
        let mut play_music = false;

        let mut volume = false;
        let mut music_volume = self.music_volume;

        let mut pitch = false;
        let mut music_pitch = self.music_pitch;

        ui.window(im_str!("Input"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((0.0, 64.0), ImGuiCond::FirstUseEver)
            .size((256.0, 192.0), ImGuiCond::FirstUseEver)
            .build(|| {
                play_sfx = ui.button(im_str!("Play Sfx"), (96.0, 32.0));
                ui.separator();

                play_music = ui.button(im_str!("Play Music"), (96.0, 32.0));
                volume = ui
                    .slider_float(im_str!("Volume"), &mut music_volume, 0.0, 1.0)
                    .build();
                pitch = ui
                    .slider_float(im_str!("Pitch"), &mut music_pitch, 0.25, 4.0)
                    .build();
            });

        if play_sfx {
            self.audio.play(self.sfx).unwrap();
        }

        if play_music {
            if let Some(source) = self.music_source {
                self.audio.stop(source);
            }

            let mut source = AudioSource::from(self.music);
            source.volume = self.music_volume;

            self.music_source = self.audio.play(source).unwrap().into();
        }

        if volume {
            self.music_volume = music_volume;

            if let Some(source) = self.music_source {
                self.audio.set_volume(source, self.music_volume);
            }
        }

        if pitch {
            self.music_pitch = music_pitch;

            if let Some(source) = self.music_source {
                self.audio.set_pitch(source, self.music_pitch);
            }
        }

        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let res = crayon_testbed::find_res_dir();

    let params = crayon_testbed::settings("CR: Audio", (256, 256));
    let mut engine = Engine::new_with(&params).unwrap();
    engine.res.mount("res", res).unwrap();

    let window = Window::new(&mut engine, params).unwrap();
    engine.run(window).unwrap();
}
