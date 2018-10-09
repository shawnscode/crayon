use std::sync::Arc;

use crayon::errors::*;
use crayon::prelude::*;
use crayon_2d::prelude::*;
use crayon_lua::prelude::*;

use console::ConsoleCanvas;

pub struct LuaWindow {
    lua: LuaSystem,
    canvas: ConsoleCanvas,
    world: Arc<World>,
    renderer: Renderer,
}

impl LuaWindow {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();

        let mut window = LuaWindow {
            canvas: ConsoleCanvas::new(&ctx, None)?,
            lua: LuaSystem::new(ctx, "res".into()),
            world: Arc::new(World::new()),
            renderer: Renderer::new(ctx.video.clone())?,
        };

        // window.lua.register(
        //     "world",
        //     ::crayon_lua::binds::modules::twod::world::namespace(window.world.clone())?,
        // );

        Ok(window)
    }

    pub fn run<T1, T2, F>(name: T1, dims: T2, func: F)
    where
        T1: Into<String>,
        T2: Into<Vector2<u32>>,
        F: Fn(&mut LuaSystem),
    {
        let res = super::find_res_dir();
        let params = super::settings(name, dims);

        let mut engine = Engine::new_with(&params).unwrap();
        engine.res.mount("res", res).unwrap();

        let mut window = LuaWindow::new(&mut engine).unwrap();
        func(&mut window.lua);

        engine.run(window).unwrap();
    }
}

// pub struct LuaRenderer<'a> {
//     renderer: &mut Renderer,
// }

impl Application for LuaWindow {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        // self.lua.update(&mut self.world, )

        // self.world.update(|e, transform| {
        //     self.debug.update(e, transform);
        //     self.spr.update(e, transform);
        //     self.litswf.update(e, transform);
        // });

        // self.world.render(|e, color_transform| {
        //     self.debug.render(&mut self.renderer, e, color_transform)
        //     self.spr.render(&mut self.renderer, e, color_transform);
        //     self.litswf.update(&mut self.renderer, e, color_transform);
        // });

        self.canvas.render(ctx);
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}
