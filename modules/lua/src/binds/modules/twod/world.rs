use std::sync::Arc;

use crayon_2d::prelude::*;
use rlua::{ExternalResult, MetaMethod, Result, String, UserData, UserDataMethods};

pub fn namespace(world: Arc<World>) -> Result<impl UserData> {
    Ok(LuaWorld { world: world })
}

struct LuaWorld {
    world: Arc<World>,
}

impl UserData for LuaWorld {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("create", |_, this, name: Option<String>| match name {
            Some(name) => Ok(LuaEntity(this.world.create(name.to_str()?))),
            None => Ok(LuaEntity(this.world.create("entity"))),
        });

        methods.add_method("delete", |_, this, e: LuaEntity| {
            this.world.delete(e.0);
            Ok(())
        });

        methods.add_method("find", |_, this, name: String| {
            Ok(this.world.find(name.to_str()?).map(|v| LuaEntity(v)))
        });

        methods.add_method("find_from", |_, this, (e, name): (LuaEntity, String)| {
            Ok(this
                .world
                .find_from(e.0, name.to_str()?)
                .map(|v| LuaEntity(v)))
        });

        methods.add_method("parent", |_, this, e: LuaEntity| {
            Ok(this.world.parent(e.0).map(|v| LuaEntity(v)))
        });

        methods.add_method(
            "set_parent",
            |_, this, (child, parent, keey_world_pose): (LuaEntity, Option<LuaEntity>, Option<bool>)| {
                this.world.set_parent(child.0, parent.map(|v| v.0), keey_world_pose.unwrap_or(false));
                Ok(())
            },
        );

        methods.add_method("is_leaf", |_, this, e: LuaEntity| {
            Ok(this.world.is_leaf(e.0))
        });

        methods.add_method("is_root", |_, this, e: LuaEntity| {
            Ok(this.world.is_root(e.0))
        });
    }
}

impl_lua_struct!(LuaEntity(Entity));
