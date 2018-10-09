#[macro_use]
pub mod utils;

pub mod input;
pub mod math;
pub mod modules;
pub mod time;
pub mod video;

use rlua::Result;

pub fn register(sys: &::LuaSystem, ctx: &::crayon::application::prelude::Context) -> Result<()> {
    sys.register("time", time::namespace(ctx.time.clone()));
    sys.register("video", video::namespace(ctx.video.clone())?);
    sys.register("input", input::namespace(sys.state(), ctx.input.clone())?);
    sys.register("math", math::namespace(sys.state())?);
    Ok(())
}
