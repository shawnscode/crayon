extern crate crayon_lua;
extern crate crayon_testbed;

use crayon_testbed::prelude::*;

fn main() {
    LuaWindow::run("CR: Lua & ImGui", (640, 480), |lua| {
        lua.create_from("res:scripts/main.lua").unwrap();
    });
}
