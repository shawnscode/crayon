use rlua::{Error, Function, Lua, Result, Table};

pub fn readonly<'a>(
    state: &'a Lua,
    tb: Table<'a>,
    call: Option<Function<'a>>,
    tostring: Option<Function<'a>>,
) -> Result<Table<'a>> {
    let metatable = state.create_table()?;
    metatable.set("__call", call)?;
    metatable.set("__tostring", tostring)?;
    metatable.set("__index", tb)?;
    metatable.set(
        "__newindex",
        state.create_function(|_, _: ()| -> Result<()> {
            Err(Error::external(format_err!(
                "attempt to update a read-only table"
            )))
        })?,
    )?;
    metatable.set("__metatable", 0)?;

    let proxy = state.create_table()?;
    proxy.set_metatable(Some(metatable));

    Ok(proxy)
}

#[macro_export]
macro_rules! impl_lua_clike_enum {
    ($name: ident($enum: tt) [ $( $field: ident ), * ] ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $name(pub $enum);

        impl $crate::rlua::UserData for $name {
            fn add_methods(methods: &mut $crate::rlua::UserDataMethods<Self>) {
                methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
                    Ok(format!("{:?}", this.0))
                });
            }
        }

        impl From<$enum> for $name {
            fn from(v: $enum) -> Self {
                $name(v)
            }
        }

        impl $name {
            fn ty<'a>(state: &'a $crate::rlua::Lua) -> $crate::rlua::Result<::rlua::Table<'a>> {
                let tb = state.create_table()?;

                $(
                    tb.set(stringify!($field), $name($enum::$field))?;
                ) *;

                let tostring = state.create_function(|_, _:()| Ok(stringify!($enum)))?;
                $crate::binds::utils::readonly(state, tb, None, Some(tostring))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_lua_struct {
    ($name: ident($enum: tt)) => {
        #[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $name(pub $enum);

        impl $crate::rlua::UserData for $name {
            fn add_methods(methods: &mut $crate::rlua::UserDataMethods<Self>) {
                methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
                    Ok(format!("{:?}", this.0))
                });
            }
        }
    };
}
