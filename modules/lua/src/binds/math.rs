use crayon::math::prelude::*;
use rlua::{Error, Lua, MetaMethod, Result, String, Table, UserData, UserDataMethods};

pub fn namespace(state: &Lua) -> Result<Table> {
    let constants = state.create_table()?;
    constants.set("Vector2f", LuaVector2f::ty(state)?)?;
    constants.set("Vector3f", LuaVector3f::ty(state)?)?;
    constants.set("Vector4f", LuaVector4f::ty(state)?)?;
    ::binds::utils::readonly(state, constants, None, None)
}

macro_rules! impl_lua_vector {
    ($name: ident($source: tt) { $( $fields: ident: $tys: ident ), * } ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(pub $source);

        impl From<$source> for $name {
            fn from(v: $source) -> Self {
                $name(v)
            }
        }

        impl Into<$source> for $name {
            fn into(self) -> $source {
                self.0
            }
        }

        impl $name {
            fn ty(state: &Lua) -> Result<Table> {
                let tb = state.create_table()?;

                tb.set(
                    "distance",
                    state.create_function(|_, (lhs, rhs): ($name, $name)| {
                        Ok(lhs.0.distance(rhs.0))
                    })?,
                )?;

                tb.set(
                    "distance2",
                    state.create_function(|_, (lhs, rhs): ($name, $name)| {
                        Ok(lhs.0.distance2(rhs.0))
                    })?,
                )?;

                tb.set(
                    "lerp",
                    state.create_function(|_, (lhs, rhs, ratio): ($name, $name, f32)| {
                        Ok($name(lhs.0.lerp(rhs.0, ratio)))
                    })?,
                )?;

                let call =
                    state.create_function(|_, (_, $($fields), * ): (Table, $($tys), * )| Ok($name($source::new($($fields), *))))?;
                let tostring = state.create_function(|_, _: ()| Ok(stringify!($source)))?;

                ::binds::utils::readonly(state, tb, Some(call), Some(tostring))
            }
        }

        impl UserData for $name {
            fn add_methods(methods: &mut UserDataMethods<Self>) {
                methods.add_meta_method(MetaMethod::Index, |_, this, k: String| {
                    let v = match k.to_str()? {
                        $(
                            stringify!($fields) => (this.0).$fields,
                        ) *
                        other => {
                            return Err(Error::external(format_err!(
                                "`index` undefined field {} of struct {}.",
                                other, stringify!($source)
                            )))
                        }
                    };

                    Ok(v)
                });

                methods.add_meta_method_mut(
                    MetaMethod::NewIndex,
                    |_, this, (k, v): (String, f32)| {
                        match k.to_str()? {
                            $(
                                stringify!($fields) => (this.0).$fields = v,
                            ) *
                            other => {
                                return Err(Error::external(format_err!(
                                    "`newindex` undefined field {} of struct {}.",
                                    other, stringify!($source)
                                )));
                            }
                        };

                        Ok(())
                    },
                );

                methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
                    Ok(format!("{:?}", this.0))
                });

                methods.add_meta_function(MetaMethod::Add, |_, (lhs, rhs): ($name, $name)| {
                    Ok($name(lhs.0 + rhs.0))
                });

                methods.add_meta_function(MetaMethod::Sub, |_, (lhs, rhs): ($name, $name)| {
                    Ok($name(lhs.0 - rhs.0))
                });

                methods.add_meta_function(MetaMethod::Mul, |_, (lhs, rhs): ($name, f32)| {
                    Ok($name(lhs.0 * rhs))
                });

                methods.add_meta_function(MetaMethod::Div, |_, (lhs, rhs): ($name, f32)| {
                    Ok($name(lhs.0 / rhs))
                });

                methods.add_method("magnitude", |_, this, _: ()| Ok(this.0.magnitude()));
                methods.add_method("magnitude2", |_, this, _: ()| Ok(this.0.magnitude2()));
            }
        }
    };
}

type Vector2f = Vector2<f32>;
impl_lua_vector!(LuaVector2f(Vector2f) { x: f32, y: f32 });

type Vector3f = Vector3<f32>;
impl_lua_vector!(LuaVector3f(Vector3f) { x: f32, y: f32, z: f32 });

type Vector4f = Vector4<f32>;
impl_lua_vector!(LuaVector4f(Vector4f) { x: f32, y: f32, z: f32, w: f32 });
