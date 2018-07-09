use crayon::prelude::*;
use crayon::video::assets::prelude::*;
use crayon_imgui::prelude::*;

use errors::*;
use utils;

struct Window {
    canvas: Canvas,
    surface: SurfaceHandle,
    info: FrameInfo,
    text: String,
    repeat_count: u32,

    click_count: u32,
    double_click_count: u32,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        let canvas = Canvas::new(ctx).unwrap();

        let mut setup = SurfaceSetup::default();
        setup.set_clear(math::Color::white(), None, None);
        setup.set_sequence(true);
        let surface = ctx.video.create_surface(setup)?;

        Ok(Window {
            canvas: canvas,
            surface: surface,
            info: Default::default(),
            text: String::new(),
            repeat_count: 0,
            click_count: 0,
            double_click_count: 0,
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let input = ctx.input.clone();

        self.text += &input.text();

        if input.is_key_repeat(event::KeyboardButton::A) {
            self.repeat_count += 1;
        }

        if input.is_mouse_click(event::MouseButton::Left) {
            self.click_count += 1;
        }

        if input.is_mouse_double_click(event::MouseButton::Left) {
            self.double_click_count += 1;
        }

        let ui = self.canvas.frame(self.surface, ctx);
        let info = self.info;
        let text = &self.text;
        let rc = self.repeat_count;
        let clicks = self.click_count;
        let double_clicks = self.double_click_count;

        ui.window(im_str!("ImGui & Crayon"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((50.0, 50.0), ImGuiCond::FirstUseEver)
            .size((400.0, 400.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("FPS: {:?}", info.fps));
                ui.text(im_str!(
                    "DrawCalls: {:?}, Triangles: {:?}",
                    info.video.drawcall,
                    info.video.triangles
                ));

                ui.text(im_str!(
                    "CPU: {:.2?}ms, GPU: {:.2?}ms",
                    utils::to_ms(info.duration),
                    utils::to_ms(info.video.duration)
                ));

                ui.separator();

                if ui.collapsing_header(im_str!("Mouse")).build() {
                    let pos = input.mouse_position();
                    let movement = input.mouse_movement();
                    let scroll = input.mouse_scroll();
                    ui.text(im_str!("Position: ({:.1},{:.1})", pos.x, pos.y));
                    ui.text(im_str!("Movement: ({:.1}, {:.1})", movement.x, movement.y));
                    ui.text(im_str!("Scroll: ({:.1}, {:.1})", scroll.x, scroll.y));

                    let is_down = input.is_mouse_down(event::MouseButton::Left);
                    let is_press = input.is_mouse_press(event::MouseButton::Left);
                    let is_release = input.is_mouse_release(event::MouseButton::Left);
                    ui.text(im_str!(
                        "Down({:?}) Pressed({:?}) Released({:?})",
                        is_down,
                        is_press,
                        is_release
                    ));

                    ui.text(im_str!(
                        "Clicks: ({:.1}, Double Clicks: {:.1})",
                        clicks,
                        double_clicks
                    ));
                };

                if ui.collapsing_header(im_str!("Keyboard")).build() {
                    let is_down = input.is_key_down(event::KeyboardButton::A);
                    let is_press = input.is_key_press(event::KeyboardButton::A);
                    let is_release = input.is_key_release(event::KeyboardButton::A);

                    ui.text(im_str!(
                        "[A] Pressed({:?}) Down({:?}) Released({:?})",
                        is_down,
                        is_press,
                        is_release
                    ));
                    ui.text(im_str!("[A] Repeat({:?})", rc));

                    let is_down = input.is_key_down(event::KeyboardButton::Z);
                    let is_press = input.is_key_press(event::KeyboardButton::Z);
                    let is_release = input.is_key_release(event::KeyboardButton::Z);
                    ui.text(im_str!(
                        "[Z] Down({:?}) Pressed({:?}) Released({:?})",
                        is_down,
                        is_press,
                        is_release
                    ));

                    ui.text_wrapped(im_str!("Text: {:?}.", text));
                };
            });

        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.info = *info;
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.size = math::Vector2::new(500, 500);

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
