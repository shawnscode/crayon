use std::fs;
use std::io::Read;
use std::sync::Arc;

use crate::sched::prelude::LockLatch;

use super::super::request::Response;
use super::super::url::Url;
use super::VFS;

#[derive(Debug, Default, Clone, Copy)]
pub struct Dir {}

impl Dir {
    pub fn new() -> Self {
        Dir {}
    }

    fn load_from(self, location: &str) -> Response {
        let mut file = fs::File::open(location)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}

impl VFS for Dir {
    fn request(&self, url: &Url, state: Arc<LockLatch<Response>>) {
        let response = self.load_from(url.path());
        state.set(response);
    }
}
