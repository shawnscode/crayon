use std::sync::Arc;

use crayon::input::prelude::*;
use rlua::{Lua, MetaMethod, RegistryKey, Result, String, Table, UserData, UserDataMethods, Value};

use binds::math::LuaVector2f;

pub fn namespace(state: &Lua, input: Arc<InputSystemShared>) -> Result<impl UserData> {
    let constants = state.create_table()?;
    constants.set("Key", LuaKey::ty(state)?)?;
    constants.set("Mouse", LuaMouseButton::ty(state)?)?;

    let constants = ::binds::utils::readonly(state, constants, None, None)?;
    Ok(LuaInput {
        input: input,
        constants: state.create_registry_value(constants)?,
    })
}

struct LuaInput {
    input: Arc<InputSystemShared>,
    constants: RegistryKey,
}

impl UserData for LuaInput {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_meta_method(
            MetaMethod::Index,
            |state, this, k: String| -> Result<Value> {
                let constants: Table = state.registry_value(&this.constants)?;
                constants.get(k)
            },
        );

        methods.add_method("is_key_down", |_, this, key: LuaKey| {
            Ok(this.input.is_key_down(key.0))
        });

        methods.add_method("is_key_press", |_, this, key: LuaKey| {
            Ok(this.input.is_key_press(key.0))
        });

        methods.add_method("is_key_release", |_, this, key: LuaKey| {
            Ok(this.input.is_key_release(key.0))
        });

        methods.add_method("is_key_repeat", |_, this, key: LuaKey| {
            Ok(this.input.is_key_repeat(key.0))
        });

        methods.add_method("text", |_, this, _: ()| Ok(this.input.text()));

        methods.add_method("is_mouse_down", |_, this, key: LuaMouseButton| {
            Ok(this.input.is_mouse_down(key.0))
        });

        methods.add_method("is_mouse_press", |_, this, key: LuaMouseButton| {
            Ok(this.input.is_mouse_press(key.0))
        });

        methods.add_method("is_mouse_release", |_, this, key: LuaMouseButton| {
            Ok(this.input.is_mouse_release(key.0))
        });

        methods.add_method("is_mouse_click", |_, this, key: LuaMouseButton| {
            Ok(this.input.is_mouse_click(key.0))
        });

        methods.add_method("is_mouse_double_click", |_, this, key: LuaMouseButton| {
            Ok(this.input.is_mouse_double_click(key.0))
        });

        methods.add_method("mouse_position", |_, this, _: ()| {
            Ok(LuaVector2f(this.input.mouse_position()))
        });

        methods.add_method("mouse_position_in_points", |_, this, _: ()| {
            Ok(LuaVector2f(this.input.mouse_position_in_points()))
        });

        methods.add_method("mouse_movement", |_, this, _: ()| {
            Ok(LuaVector2f(this.input.mouse_movement()))
        });

        methods.add_method("mouse_movement_in_points", |_, this, _: ()| {
            Ok(LuaVector2f(this.input.mouse_movement_in_points()))
        });

        methods.add_method("mouse_scroll", |_, this, _: ()| {
            Ok(LuaVector2f(this.input.mouse_scroll()))
        });

        methods.add_method("is_finger_touched", |_, this, n: usize| {
            Ok(this.input.is_finger_touched(n))
        });

        methods.add_method("finger_position", |_, this, n: usize| {
            Ok(this.input.finger_position(n).map(|v| LuaVector2f(v)))
        });

        methods.add_method("finger_position_in_points", |_, this, n: usize| {
            Ok(this
                .input
                .finger_position_in_points(n)
                .map(|v| LuaVector2f(v)))
        });
    }
}

impl_lua_clike_enum!(LuaKey(Key) [
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15,
    Snapshot, Scroll, Pause, Insert, Home, Delete, End, PageDown, PageUp,
    Left, Up, Right, Down,
    Back, Return, Space, Compose, Caret,
    Numlock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    Add, Backslash, Calculator, Capital, Colon, Comma, Convert, Decimal, Divide, Equals,
    LAlt, LBracket, LControl, LShift, LWin, Minus, Multiply, Mute, NavigateForward, NavigateBackward,
    NumpadComma, NumpadEnter, NumpadEquals, Period, PlayPause, Power, PrevTrack,
    RAlt, RBracket, RControl, RShift, RWin, Semicolon, Slash, Sleep, Stop, Subtract, Tab,
    Underline, Unlabeled, VolumeDown, VolumeUp, Wake
]);

impl_lua_clike_enum!(LuaMouseButton(MouseButton) [ Left, Right, Middle ]);
