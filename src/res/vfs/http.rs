use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use errors::*;

use super::super::request::{RequestState, Response};
use super::super::url::Url;
use super::VFS;

pub struct Http {}

impl Http {
    pub fn new() -> Self {
        Http {}
    }

    fn load_from(&self, location: &str) -> Response {
        let mut file = fs::File::open(location)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}

impl VFS for Http {
    fn request(&self, url: Url, state: Arc<Mutex<RequestState>>) {
        let response = self.load_from(url.path());
        *state.lock().unwrap() = RequestState::Ok(response);
    }
}
