use application::time::Instant;
use wasm_bindgen::prelude::*;

use errors::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log::set_boxed_logger(Box::new(WebBrowserLogger {})).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}

pub fn instant() -> Instant {
    let ms = web_sys::window()
        .expect("should have a window in this context")
        .performance()
        .expect("performance should be available")
        .now();

    Instant::from_millis(ms as u64)
}

struct WebBrowserLogger;

impl ::log::Log for WebBrowserLogger {
    fn enabled(&self, metadata: &::log::Metadata) -> bool {
        metadata.level() <= ::log::Level::Info
    }

    fn log(&self, record: &::log::Record) {
        if self.enabled(record.metadata()) {
            let filename = record.file().unwrap_or("Unknown");

            log(&format!(
                "{}: {} ({}:{})",
                record.level(),
                record.args(),
                filename,
                record.line().unwrap_or(0)
            ));
        }
    }

    fn flush(&self) {}
}

use application::events;
use application::prelude::{Application, Engine};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

pub struct WasmEngine<T: Application> {
    core: Engine,
    application: T,
    closure: Option<Closure<Fn(f64)>>,
}

impl<T: 'static + Application> WasmEngine<T> {
    pub fn new(core: Engine, application: T) -> Rc<RefCell<WasmEngine<T>>> {
        let wasm = WasmEngine {
            core: core,
            application: application,
            closure: None,
        };

        let wasm = Rc::new(RefCell::new(wasm));
        let clone = wasm.clone();
        wasm.borrow_mut().closure = Some(Closure::wrap(Box::new(move |time: f64| {
            clone.borrow_mut().run().unwrap();
        })));

        wasm
    }

    pub fn run(&mut self) -> Result<()> {
        let mut alive = true;

        self.core.time.advance(false);

        self.application.on_update(&self.core.context)?;
        self.application.on_render(&self.core.context)?;

        self.core.input.advance(self.core.window.hidpi());
        // Poll any possible events first.
        for v in self.core.window.advance() {
            match *v {
                events::Event::Application(value) => {
                    self.application
                        .on_receive_event(&self.core.context, value)?;

                    if let events::ApplicationEvent::Closed = value {
                        alive = false;
                    }
                }

                events::Event::InputDevice(value) => self.core.input.update_with(value),
            }
        }

        if !alive || self.core.context.is_shutdown() || self.core.headless {
            self.application.on_exit(&self.core.context)?;
            return Ok(());
        }

        self.core.video.swap_frames();
        self.core.video.advance(&self.core.window)?;

        if let Some(ref closure) = self.closure {
            web_sys::window()
                .expect("should have a window in this context")
                .request_animation_frame(closure.as_ref().unchecked_ref())
                .unwrap();
        }

        Ok(())
    }
}
