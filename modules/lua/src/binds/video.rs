use std::sync::Arc;

use crayon::video::prelude::*;
use rlua::{MetaMethod, Result, String, UserData, UserDataMethods};

pub fn namespace(video: Arc<VideoSystemShared>) -> Result<impl UserData> {
    Ok(LuaVideo { video: video })
}

struct LuaVideo {
    video: Arc<VideoSystemShared>,
}

impl UserData for LuaVideo {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("create_mesh_from", |_, this, location: String| {
            let location = location.to_str()?;
            let handle = this.video.create_mesh_from(location).unwrap();
            Ok(LuaMeshHandle(handle))
        });

        methods.add_method("delete_mesh", |_, this, handle: LuaMeshHandle| {
            this.video.delete_mesh(handle.0);
            Ok(())
        });

        methods.add_method("create_texture_from", |_, this, location: String| {
            let location = location.to_str()?;
            let handle = this.video.create_texture_from(location).unwrap();
            Ok(LuaTextureHandle(handle))
        });

        methods.add_method("delete_texture", |_, this, handle: LuaTextureHandle| {
            this.video.delete_texture(handle.0);
            Ok(())
        });
    }
}

impl_lua_struct!(LuaTextureHandle(TextureHandle));
impl_lua_struct!(LuaMeshHandle(MeshHandle));
