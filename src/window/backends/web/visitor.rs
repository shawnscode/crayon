use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    self, Document, Element, HtmlCanvasElement, KeyboardEvent, MouseEvent, Node, UiEvent, Window,
};

use crate::input::prelude::{InputEvent, MouseButton};
use crate::window::prelude::{Event, WindowEvent, WindowParams};

use crate::math::prelude::Vector2;
use crate::errors::*;

use super::{types, Visitor};

#[allow(dead_code)]
pub struct WebVisitor {
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
    events: Arc<Mutex<Vec<Event>>>,
    on_mouse_move: Closure<FnMut(MouseEvent)>,
    on_mouse_down: Closure<FnMut(MouseEvent)>,
    on_mouse_up: Closure<FnMut(MouseEvent)>,
    on_key_down: Closure<FnMut(KeyboardEvent)>,
    on_key_up: Closure<FnMut(KeyboardEvent)>,
    on_resize: Closure<FnMut(UiEvent)>,
    on_focus: Closure<FnMut(UiEvent)>,
    on_lost_focus: Closure<FnMut(UiEvent)>,
}

impl WebVisitor {
    pub fn new(params: WindowParams) -> Result<Self> {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        let header = document.create_element("p").unwrap();
        header.set_inner_html(&params.title);
        AsRef::<Node>::as_ref(&body)
            .append_child(header.as_ref())
            .unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        canvas
            .unchecked_ref::<Element>()
            .set_attribute("id", "canvas")
            .unwrap();

        AsRef::<Node>::as_ref(&body)
            .append_child(canvas.as_ref())
            .unwrap();

        let events = Arc::new(Mutex::new(Vec::new()));

        let on_mouse_down = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |v: MouseEvent| {
                let button = match v.button() {
                    0 => MouseButton::Left,
                    1 => MouseButton::Middle,
                    2 => MouseButton::Right,
                    i => MouseButton::Other(i as u8),
                };

                let evt = Event::InputDevice(InputEvent::MousePressed { button });
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())
            .unwrap();

        let on_mouse_up = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |v: MouseEvent| {
                let button = match v.button() {
                    0 => MouseButton::Left,
                    1 => MouseButton::Middle,
                    2 => MouseButton::Right,
                    i => MouseButton::Other(i as u8),
                };

                let evt = Event::InputDevice(InputEvent::MouseReleased { button });
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())
            .unwrap();

        let on_mouse_move = {
            let clone = events.clone();
            let window = window.clone();
            let canvas = canvas.clone();
            Closure::wrap(Box::new(move |v: MouseEvent| {
                let dpr = window.device_pixel_ratio() as f32;
                let height = canvas.height() as f32 / dpr;
                let rect = canvas.get_bounding_client_rect();

                let position = (
                    v.layer_x() as f32 - rect.x() as f32,
                    height - v.layer_y() as f32 + rect.y() as f32,
                );

                let evt = Event::InputDevice(InputEvent::MouseMoved { position });
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())
            .unwrap();

        let on_key_down = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |v: KeyboardEvent| {
                if let Some(key) = types::from_virtual_key_code(&v.key()) {
                    let evt = Event::InputDevice(InputEvent::KeyboardPressed { key });
                    clone.lock().unwrap().push(evt);
                }
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())
            .unwrap();

        let on_key_up = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |v: KeyboardEvent| {
                if let Some(key) = types::from_virtual_key_code(&v.key()) {
                    let evt = Event::InputDevice(InputEvent::KeyboardReleased { key });
                    clone.lock().unwrap().push(evt);
                }
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("keyup", on_key_up.as_ref().unchecked_ref())
            .unwrap();

        let on_focus = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |_: UiEvent| {
                let evt = Event::Window(WindowEvent::GainFocus);
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("focus", on_focus.as_ref().unchecked_ref())
            .unwrap();

        let on_lost_focus = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |_: UiEvent| {
                let evt = Event::Window(WindowEvent::GainFocus);
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("blur", on_lost_focus.as_ref().unchecked_ref())
            .unwrap();

        let on_resize = {
            let clone = events.clone();
            let canvas = canvas.clone();
            Closure::wrap(Box::new(move |_: UiEvent| {
                let evt = Event::Window(WindowEvent::Resized(canvas.width(), canvas.height()));
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };

        canvas
            .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
            .unwrap();

        let visitor = WebVisitor {
            window: window,
            document: document,
            canvas: canvas,
            events: events,
            on_mouse_down: on_mouse_down,
            on_mouse_up: on_mouse_up,
            on_mouse_move: on_mouse_move,
            on_key_down: on_key_down,
            on_key_up: on_key_up,
            on_focus: on_focus,
            on_lost_focus: on_lost_focus,
            on_resize: on_resize,
        };

        let dpr = visitor.device_pixel_ratio();
        let dims = Vector2::new(
            (params.size.x as f32 * dpr) as u32,
            (params.size.y as f32 * dpr) as u32,
        );

        visitor.resize(dims);
        Ok(visitor)
    }
}

impl Visitor for WebVisitor {
    #[inline]
    fn show(&self) {
        warn!("It does not make sense to `show` window in browser.")
    }

    #[inline]
    fn hide(&self) {
        warn!("It does not make sense to `hide` window in browser.")
    }

    #[inline]
    fn position(&self) -> Vector2<i32> {
        (0, 0).into()
    }

    #[inline]
    fn dimensions(&self) -> Vector2<u32> {
        let dpr = self.window.device_pixel_ratio() as f32;
        Vector2::new(
            (self.canvas.width() as f32 / dpr) as u32,
            (self.canvas.height() as f32 / dpr) as u32,
        )
    }

    #[inline]
    fn device_pixel_ratio(&self) -> f32 {
        self.window.device_pixel_ratio() as f32
    }

    #[inline]
    fn resize(&self, dims: Vector2<u32>) {
        self.canvas.set_width(dims.x);
        self.canvas.set_height(dims.y);

        let dpr = self.device_pixel_ratio();
        self.canvas
            .unchecked_ref::<Element>()
            .set_attribute(
                "style",
                &format!(
                    "width: {}px; height: {}px;",
                    (dims.x as f32 / dpr) as u32,
                    (dims.y as f32 / dpr) as u32
                ),
            ).unwrap();
    }

    #[inline]
    fn poll_events(&mut self, v: &mut Vec<Event>) {
        let mut events = self.events.lock().unwrap();
        v.extend(events.drain(..));
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.document.has_focus().unwrap_or(false)
    }

    #[inline]
    fn make_current(&self) -> Result<()> {
        warn!("You can not `make_current` in browser.");
        Ok(())
    }

    #[inline]
    fn swap_buffers(&self) -> Result<()> {
        Ok(())
    }
}
