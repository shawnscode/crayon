#[macro_use]
extern crate crayon_testbed;
use crayon_testbed::prelude::*;
use crayon_testbed::ImGuiCond;

struct Window {
    canvas: ConsoleCanvas,
    text: String,
    repeat_count: u32,
    click_count: u32,
    double_click_count: u32,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        Ok(Window {
            canvas: ConsoleCanvas::new(&ctx, Color::white())?,
            text: String::new(),
            repeat_count: 0,
            click_count: 0,
            double_click_count: 0,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let input = ctx.input.clone();

        self.text += &input.text();

        if input.is_key_repeat(Key::A) {
            self.repeat_count += 1;
        }

        if input.is_mouse_click(MouseButton::Left) {
            self.click_count += 1;
        }

        if input.is_mouse_double_click(MouseButton::Left) {
            self.double_click_count += 1;
        }

        let ui = self.canvas.render(ctx);
        let text = &self.text;
        let rc = self.repeat_count;
        let clicks = self.click_count;
        let double_clicks = self.double_click_count;

        ui.window(im_str!("Input"))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .position((100.0, 100.0), ImGuiCond::FirstUseEver)
            .size((400.0, 400.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.collapsing_header(im_str!("Mouse")).build() {
                    let pos = input.mouse_position_in_points();
                    let movement = input.mouse_movement_in_points();
                    let scroll = input.mouse_scroll_in_points();
                    ui.text(im_str!("Position: ({:.1},{:.1})", pos.x, pos.y));
                    ui.text(im_str!("Movement: ({:.1}, {:.1})", movement.x, movement.y));
                    ui.text(im_str!("Scroll: ({:.1}, {:.1})", scroll.x, scroll.y));

                    let is_down = input.is_mouse_down(MouseButton::Left);
                    let is_press = input.is_mouse_press(MouseButton::Left);
                    let is_release = input.is_mouse_release(MouseButton::Left);
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
                    let is_down = input.is_key_down(Key::A);
                    let is_press = input.is_key_press(Key::A);
                    let is_release = input.is_key_release(Key::A);

                    ui.text(im_str!(
                        "[A] Pressed({:?}) Down({:?}) Released({:?})",
                        is_down,
                        is_press,
                        is_release
                    ));
                    ui.text(im_str!("[A] Repeat({:?})", rc));

                    let is_down = input.is_key_down(Key::Z);
                    let is_press = input.is_key_press(Key::Z);
                    let is_release = input.is_key_release(Key::Z);
                    ui.text(im_str!(
                        "[Z] Down({:?}) Pressed({:?}) Released({:?})",
                        is_down,
                        is_press,
                        is_release
                    ));

                    ui.text_wrapped(im_str!("Text: {:?}.", text));
                };

                if ui.collapsing_header(im_str!("Touch Pad")).build() {}
            });

        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let params = crayon_testbed::settings("CR: Input", (600, 600));
    let mut engine = Engine::new_with(&params).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
