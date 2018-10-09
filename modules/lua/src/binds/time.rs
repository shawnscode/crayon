use std::sync::Arc;

use crayon::application::time::TimeSystemShared;
use rlua::{UserData, UserDataMethods};

pub fn namespace(time: Arc<TimeSystemShared>) -> impl UserData {
    LuaTime(time)
}

struct LuaTime(Arc<TimeSystemShared>);

impl UserData for LuaTime {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("fps", |_, v, _: ()| Ok(v.0.get_fps()));

        methods.add_method("frame_delta", |_, v, _: ()| {
            let duration = v.0.frame_delta();
            let secs = duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;
            Ok(secs)
        });

        methods.add_method("frame_delta_ms", |_, v, _: ()| {
            let duration = v.0.frame_delta();
            let secs = duration.as_secs() as f32 * 1000.0 + duration.subsec_millis() as f32;
            Ok(secs)
        });
    }
}
