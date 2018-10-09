use std::sync::Arc;

use crayon_audio::prelude::*;
use rlua::{ExternalResult, MetaMethod, Result, String, UserData, UserDataMethods};

pub fn namespace(audio: Arc<AudioSystemShared>) -> Result<impl UserData> {
    Ok(LuaAudio { audio: audio })
}

struct LuaAudio {
    audio: Arc<AudioSystemShared>,
}

impl UserData for LuaAudio {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("create_clip_from", |_, this, location: String| {
            let location = location.to_str()?;
            let handle = this.audio.create_clip_from(location).to_lua_err()?;
            Ok(LuaAudioClipHandle(handle))
        });

        methods.add_method("delete_mesh", |_, this, handle: LuaAudioClipHandle| {
            this.audio.delete_clip(handle.0);
            Ok(())
        });

        methods.add_method("play", |_, this, handle: LuaAudioClipHandle| {
            let handle = this.audio.play(handle.0).to_lua_err()?;
            Ok(LuaAudioSourceHandle(handle))
        });

        methods.add_method("stop", |_, this, handle: LuaAudioSourceHandle| {
            this.audio.stop(handle.0);
            Ok(())
        });

        methods.add_method(
            "set_volume",
            |_, this, (handle, volume): (LuaAudioSourceHandle, f32)| {
                this.audio.set_volume(handle.0, volume);
                Ok(())
            },
        );

        methods.add_method(
            "set_pitch",
            |_, this, (handle, pitch): (LuaAudioSourceHandle, f32)| {
                this.audio.set_pitch(handle.0, pitch);
                Ok(())
            },
        );
    }
}

impl_lua_struct!(LuaAudioClipHandle(AudioClipHandle));
impl_lua_struct!(LuaAudioSourceHandle(AudioSourceHandle));
