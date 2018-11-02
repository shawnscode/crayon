use application::time::Instant;
use wasm_bindgen::prelude::*;

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
