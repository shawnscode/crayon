use std::ops::{Deref, DerefMut};

use crayon::application::prelude::*;
use crayon::errors::*;
use crayon::input::prelude::*;
use crayon::video::prelude::*;

use imgui;
use renderer::Renderer;

pub struct FrameGuard<'a> {
    renderer: &'a mut Renderer,
    frame: Option<imgui::Ui<'a>>,
    surface: Option<SurfaceHandle>,
}

impl<'a> Deref for FrameGuard<'a> {
    type Target = imgui::Ui<'a>;

    fn deref(&self) -> &Self::Target {
        self.frame.as_ref().unwrap()
    }
}

impl<'a> DerefMut for FrameGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.frame.as_mut().unwrap()
    }
}

impl<'a> Drop for FrameGuard<'a> {
    fn drop(&mut self) {
        if let Some(ui) = self.frame.take() {
            self.renderer.draw(self.surface, ui).unwrap();
        }
    }
}

pub struct Canvas {
    ctx: imgui::ImGui,
    renderer: Renderer,
}

impl Canvas {
    pub fn new(ctx: &Context) -> Result<Self> {
        let mut imgui = imgui::ImGui::init();
        imgui.set_ini_filename(None);

        let renderer = Renderer::new(ctx, &mut imgui)?;

        Self::bind_keycode(&mut imgui);
        Ok(Canvas {
            ctx: imgui,
            renderer: renderer,
        })
    }

    pub fn frame<T>(&mut self, ctx: &Context, surface: T) -> FrameGuard
    where
        T: Into<Option<SurfaceHandle>>,
    {
        // Update input device states.
        Self::update_mouse_state(&mut self.ctx, &ctx);
        Self::update_keycode_state(&mut self.ctx, &ctx.input);

        // Generates frame builder.
        let duration = ctx.time.frame_delta();
        let ts = duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;

        FrameGuard {
            renderer: &mut self.renderer,
            frame: Some(self.ctx.frame(
                ctx.window.dimensions_in_points().into(),
                ctx.window.dimensions().into(),
                ts,
            )),
            surface: surface.into(),
        }
    }

    fn bind_keycode(imgui: &mut imgui::ImGui) {
        use imgui::ImGuiKey;

        imgui.set_imgui_key(ImGuiKey::Tab, 0);
        imgui.set_imgui_key(ImGuiKey::LeftArrow, 1);
        imgui.set_imgui_key(ImGuiKey::RightArrow, 2);
        imgui.set_imgui_key(ImGuiKey::UpArrow, 3);
        imgui.set_imgui_key(ImGuiKey::DownArrow, 4);
        imgui.set_imgui_key(ImGuiKey::PageUp, 5);
        imgui.set_imgui_key(ImGuiKey::PageDown, 6);
        imgui.set_imgui_key(ImGuiKey::Home, 7);
        imgui.set_imgui_key(ImGuiKey::End, 8);
        imgui.set_imgui_key(ImGuiKey::Delete, 9);
        imgui.set_imgui_key(ImGuiKey::Backspace, 10);
        imgui.set_imgui_key(ImGuiKey::Enter, 11);
        imgui.set_imgui_key(ImGuiKey::Escape, 12);
        imgui.set_imgui_key(ImGuiKey::A, 13);
        imgui.set_imgui_key(ImGuiKey::C, 14);
        imgui.set_imgui_key(ImGuiKey::V, 15);
        imgui.set_imgui_key(ImGuiKey::X, 16);
        imgui.set_imgui_key(ImGuiKey::Y, 17);
        imgui.set_imgui_key(ImGuiKey::Z, 18);
    }

    fn update_keycode_state(imgui: &mut imgui::ImGui, input: &InputSystemShared) {
        imgui.set_key(0, input.is_key_down(Key::Tab));
        imgui.set_key(1, input.is_key_down(Key::Left));
        imgui.set_key(2, input.is_key_down(Key::Right));
        imgui.set_key(3, input.is_key_down(Key::Up));
        imgui.set_key(4, input.is_key_down(Key::Down));
        imgui.set_key(5, input.is_key_down(Key::PageUp));
        imgui.set_key(6, input.is_key_down(Key::PageDown));
        imgui.set_key(7, input.is_key_down(Key::Home));
        imgui.set_key(8, input.is_key_down(Key::End));
        imgui.set_key(9, input.is_key_down(Key::Delete));
        imgui.set_key(10, input.is_key_down(Key::Back));
        imgui.set_key(11, input.is_key_down(Key::Return));
        imgui.set_key(12, input.is_key_down(Key::Escape));
        imgui.set_key(13, input.is_key_down(Key::A));
        imgui.set_key(14, input.is_key_down(Key::C));
        imgui.set_key(15, input.is_key_down(Key::V));
        imgui.set_key(16, input.is_key_down(Key::X));
        imgui.set_key(17, input.is_key_down(Key::Y));
        imgui.set_key(18, input.is_key_down(Key::Z));

        let lcontrol = input.is_key_down(Key::LControl);
        let rcontrol = input.is_key_down(Key::RControl);
        imgui.set_key_ctrl(lcontrol || rcontrol);

        let lshift = input.is_key_down(Key::LShift);
        let rshift = input.is_key_down(Key::RShift);
        imgui.set_key_shift(lshift || rshift);

        let lalt = input.is_key_down(Key::LAlt);
        let ralt = input.is_key_down(Key::RAlt);
        imgui.set_key_alt(lalt || ralt);

        let lwin = input.is_key_down(Key::LWin);
        let rwin = input.is_key_down(Key::RWin);
        imgui.set_key_super(lwin || rwin);
    }

    fn update_mouse_state(imgui: &mut imgui::ImGui, ctx: &Context) {
        let dims = ctx.window.dimensions_in_points();
        let pos = ctx.input.mouse_position_in_points();
        imgui.set_mouse_pos(pos.x, dims.y as f32 - pos.y);

        let l = ctx.input.is_mouse_down(MouseButton::Left);
        let r = ctx.input.is_mouse_down(MouseButton::Right);
        let m = ctx.input.is_mouse_down(MouseButton::Middle);
        imgui.set_mouse_down(&[l, r, m, false, false]);

        imgui.set_mouse_wheel(ctx.input.mouse_scroll().y);
    }
}
