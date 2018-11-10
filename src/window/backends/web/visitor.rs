use wasm_bindgen::JsCast;
use web_sys::{self, Document, Element, HtmlCanvasElement, Node, Window};

use window::events::Event;
use window::WindowParams;

use errors::*;
use math::prelude::Vector2;

use super::Visitor;

pub struct WebVisitor {
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
}

impl WebVisitor {
    pub fn new(params: WindowParams) -> Result<Self> {
        info!("Creates WebVisitor.");

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

        let visitor = WebVisitor {
            window: window,
            document: document,
            canvas: canvas,
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
    fn poll_events(&mut self, _: &mut Vec<Event>) {}

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
