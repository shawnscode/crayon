#[cfg(not(target_arch = "wasm32"))]
pub mod dir;
#[cfg(target_arch = "wasm32")]
pub mod http;

use std::sync::{Arc, Mutex};

use super::request::RequestState;
use super::url::Url;

pub trait VFS: Send + Sync + 'static {
    fn request(&self, url: Url, state: Arc<Mutex<RequestState>>);
}
