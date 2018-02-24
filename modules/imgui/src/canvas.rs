use std::ops::{Deref, DerefMut};

use imgui;
use crayon::{application, graphics, input};
use crayon::application::Result;
use renderer::Renderer;

pub struct FrameGuard<'a> {
    renderer: &'a mut Renderer,
    frame: Option<imgui::Ui<'a>>,
    surface: graphics::SurfaceHandle,
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
            self.renderer.render(self.surface, ui).unwrap();
        }
    }
}

pub struct Canvas {
    ctx: imgui::ImGui,
    renderer: Renderer,
}

impl Canvas {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let mut imgui = imgui::ImGui::init();
        imgui.set_ini_filename(None);

        let renderer = Renderer::new(ctx, &mut imgui)?;

        Self::bind_keycode(&mut imgui);
        Ok(Canvas {
            ctx: imgui,
            renderer: renderer,
        })
    }

    pub fn frame<'a>(
        &'a mut self,
        surface: graphics::SurfaceHandle,
        ctx: &application::Context,
    ) -> FrameGuard<'a> {
        // Update input device states.
        let input = ctx.shared::<input::InputSystem>();
        Self::update_mouse_state(&mut self.ctx, &input);
        Self::update_keycode_state(&mut self.ctx, &input);

        // Generates frame builder.
        let v = ctx.shared::<graphics::GraphicsSystem>();
        let duration = ctx.shared::<application::TimeSystem>().frame_delta();
        let ts = duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;

        let (dp, d) = (v.dimensions_in_pixels(), v.dimensions());

        FrameGuard {
            renderer: &mut self.renderer,
            frame: Some(self.ctx.frame(d, dp, ts)),
            surface: surface,
        }
    }

    pub fn render(&mut self) {}

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

    fn update_keycode_state(imgui: &mut imgui::ImGui, input: &input::InputSystemShared) {
        use self::application::event::KeyboardButton;

        imgui.set_key(0, input.is_key_down(KeyboardButton::Tab));
        imgui.set_key(1, input.is_key_down(KeyboardButton::Left));
        imgui.set_key(2, input.is_key_down(KeyboardButton::Right));
        imgui.set_key(3, input.is_key_down(KeyboardButton::Up));
        imgui.set_key(4, input.is_key_down(KeyboardButton::Down));
        imgui.set_key(5, input.is_key_down(KeyboardButton::PageUp));
        imgui.set_key(6, input.is_key_down(KeyboardButton::PageDown));
        imgui.set_key(7, input.is_key_down(KeyboardButton::Home));
        imgui.set_key(8, input.is_key_down(KeyboardButton::End));
        imgui.set_key(9, input.is_key_down(KeyboardButton::Delete));
        imgui.set_key(10, input.is_key_down(KeyboardButton::Back));
        imgui.set_key(11, input.is_key_down(KeyboardButton::Return));
        imgui.set_key(12, input.is_key_down(KeyboardButton::Escape));
        imgui.set_key(13, input.is_key_down(KeyboardButton::A));
        imgui.set_key(14, input.is_key_down(KeyboardButton::C));
        imgui.set_key(15, input.is_key_down(KeyboardButton::V));
        imgui.set_key(16, input.is_key_down(KeyboardButton::X));
        imgui.set_key(17, input.is_key_down(KeyboardButton::Y));
        imgui.set_key(18, input.is_key_down(KeyboardButton::Z));

        let lcontrol = input.is_key_down(KeyboardButton::LControl);
        let rcontrol = input.is_key_down(KeyboardButton::RControl);
        imgui.set_key_ctrl(lcontrol || rcontrol);

        let lshift = input.is_key_down(KeyboardButton::LShift);
        let rshift = input.is_key_down(KeyboardButton::RShift);
        imgui.set_key_shift(lshift || rshift);

        let lalt = input.is_key_down(KeyboardButton::LAlt);
        let ralt = input.is_key_down(KeyboardButton::RAlt);
        imgui.set_key_alt(lalt || ralt);

        let lwin = input.is_key_down(KeyboardButton::LWin);
        let rwin = input.is_key_down(KeyboardButton::RWin);
        imgui.set_key_super(lwin || rwin);
    }

    fn update_mouse_state(imgui: &mut imgui::ImGui, input: &input::InputSystemShared) {
        use self::application::event::MouseButton;

        let pos = input.mouse_position();
        imgui.set_mouse_pos(pos.x, pos.y);

        let l = input.is_mouse_down(MouseButton::Left);
        let r = input.is_mouse_down(MouseButton::Right);
        let m = input.is_mouse_down(MouseButton::Middle);
        imgui.set_mouse_down(&[l, r, m, false, false]);

        imgui.set_mouse_wheel(input.mouse_scroll().y);
    }
}
