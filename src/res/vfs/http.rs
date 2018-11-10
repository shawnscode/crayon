use std::fs;
use std::io::Read;
use std::sync::Arc;

use sched::prelude::LockLatch;

use super::super::request::Response;
use super::super::url::Url;
use super::VFS;

#[derive(Debug, Clone, Copy)]
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
    fn request(&self, url: &Url, state: Arc<LockLatch<Response>>) {
        let response = self.load_from(url.path());
        state.set(response);
    }
}
