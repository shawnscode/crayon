use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils::time::Timestamp;

pub fn timestamp() -> Timestamp {
    let ms = web_sys::window()
        .expect("should have a window in this context")
        .performance()
        .expect("performance should be available")
        .now();

    Timestamp::from_millis(ms as u64)
}

pub(crate) fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log::set_boxed_logger(Box::new(WebBrowserLogger {})).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}

pub(crate) fn run_forever<F, F2>(mut advance: F, mut finished: F2) -> Result<(), failure::Error>
where
    F: FnMut() -> Result<bool, failure::Error> + 'static,
    F2: FnMut() -> Result<(), failure::Error> + 'static,
{
    let closure: Rc<RefCell<Option<Closure<FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let clone = closure.clone();

    *closure.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
        if advance().unwrap() {
            if let Some(inner) = clone.borrow().as_ref() {
                web_sys::window()
                    .expect("should have a window in this context")
                    .request_animation_frame(inner.as_ref().unchecked_ref())
                    .unwrap();
            }
        } else {
            finished().unwrap();
        }
    })));

    if let Some(inner) = closure.borrow().as_ref() {
        web_sys::window()
            .expect("should have a window in this context")
            .request_animation_frame(inner.as_ref().unchecked_ref())
            .unwrap();
    }

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
