#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
#[cfg(feature = "twod")]
extern crate crayon_2d;
#[cfg(feature = "audio")]
extern crate crayon_audio;

pub extern crate rlua;

pub mod assets;
pub mod binds;

pub mod prelude {
    pub use {LuaSystem, ScriptHandle};
}

use std::sync::Arc;

use crayon::application::prelude::*;
use crayon::errors::*;
use crayon::res::prelude::*;
use crayon::utils::ObjectPool;

use rlua::{
    Error, Function, Lua, RegistryKey, Result as LuaResult, String as LuaString, Table, ToLua,
    Value,
};

use self::assets::bytecode::BytecodeHandle;
use self::assets::bytecode_loader::BytecodeLoader;

impl_handle!(ScriptHandle);

pub struct LuaSystem {
    state: Lua,
    bytecodes: Arc<Registry<BytecodeHandle, BytecodeLoader>>,
    index: RegistryKey,
    scripts: ObjectPool<ScriptHandle, RegistryKey>,
}

impl LuaSystem {
    pub fn new(ctx: &Context, vfs: String) -> Self {
        let state = Lua::new();
        let bytecodes = Arc::new(Registry::new(ctx.res.clone(), BytecodeLoader::new()));

        let index = {
            let registry = bytecodes.clone();

            // Add seacher for custom loader.
            let s = state
                .create_function(move |state, name: LuaString| {
                    let name = name.to_str()?;
                    let name = name.replace('.', "/");

                    let location = if name.ends_with(".lua") {
                        format!("{}:{}", vfs, name)
                    } else {
                        format!("{}:{}.lua", vfs, name)
                    };

                    if let Ok(handle) = registry.create_from(location.as_ref()) {
                        let bc = registry.wait_and(handle, |v| v.clone()).unwrap();
                        registry.delete(handle);
                        Ok(state.load(&bc.0, None))
                    } else {
                        Err(Error::external(format_err!(
                            "failed to load bytecode from {}",
                            name
                        )))
                    }
                }).unwrap();

            let g = state.globals();
            let p: Table = g.get("package").unwrap();
            let searchers: Table = p.get("searchers").unwrap();

            // Shifts table elements.
            for i in (2..(searchers.raw_len() + 1)).rev() {
                searchers
                    .raw_set::<_, Function>(i, searchers.raw_get(i - 1).unwrap())
                    .unwrap();
            }

            searchers.raw_set(1, s).unwrap();

            // Create context table.
            let tb = state.create_table().unwrap();
            state.create_registry_value(tb).unwrap()
        };

        let sys = LuaSystem {
            state: state,
            bytecodes: bytecodes,
            index: index,
            scripts: ObjectPool::new(),
        };

        binds::register(&sys, ctx).unwrap();
        sys
    }

    #[inline]
    pub fn state(&self) -> &Lua {
        &self.state
    }

    #[inline]
    pub fn register<'a, T: 'a>(&'a self, name: &str, value: T)
    where
        T: ToLua<'a>,
    {
        let ctx: Table = self.state.registry_value(&self.index).unwrap();
        ctx.set(name, value).unwrap();
    }

    #[inline]
    pub fn create_from<'a, T>(&'a mut self, location: T) -> Result<ScriptHandle>
    where
        T: Into<Location<'a>>,
    {
        let location = location.into();
        let handle = self.bytecodes.create_from(location)?;
        let bc = self.bytecodes.wait_and(handle, |v| v.clone()).unwrap();
        self.bytecodes.delete(handle);

        self.create_from_source(&bc.0, location.filename())
    }

    #[inline]
    pub fn create_from_source<'a, T, T2>(&mut self, src: T, name: T2) -> Result<ScriptHandle>
    where
        T: AsRef<str>,
        T2: Into<Option<&'a str>>,
    {
        let tb: Table = self.state.exec(src.as_ref(), name.into()).unwrap();
        let ctx: Table = self.state.registry_value(&self.index).unwrap();
        call(ctx, tb.clone(), "ctor");

        let index = self.state.create_registry_value(tb).unwrap();
        Ok(self.scripts.create(index))
    }

    #[inline]
    pub fn free(&mut self, handle: ScriptHandle) {
        if let Some(index) = self.scripts.free(handle) {
            let tb: Table = self.state.registry_value(&index).unwrap();
            let ctx: Table = self.state.registry_value(&self.index).unwrap();
            call(ctx, tb, "dtor");

            self.state.remove_registry_value(index).unwrap();
        }
    }

    #[inline]
    pub fn update(&self) {
        let ctx: Table = self.state.registry_value(&self.index).unwrap();
        for v in self.scripts.values() {
            let behaviour: Table = self.state.registry_value(v).unwrap();
            call(ctx.clone(), behaviour, "on_update");
        }
    }
}

#[inline]
fn call<'lua>(ctx: Table<'lua>, tb: Table<'lua>, method: &str) {
    let func: LuaResult<Function> = tb.get(method);
    if let Ok(func) = func {
        let rsp: LuaResult<Value> = func.call((tb, ctx));
        match rsp {
            Err(Error::CallbackError {
                ref traceback,
                ref cause,
            }) => {
                error!("callback error: {}. {}", cause, traceback);
            }
            Err(error) => {
                error!("{}", error);
            }
            Ok(_) => {}
        }
    }
}
