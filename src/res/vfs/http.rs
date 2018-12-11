use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{XmlHttpRequest, XmlHttpRequestResponseType};

use crate::sched::prelude::LockLatch;

use super::super::request::Response;
use super::super::url::Url;
use super::VFS;

#[derive(Debug, Clone, Copy)]
pub struct Http {}

impl Http {
    pub fn new() -> Self {
        Http {}
    }
}

impl VFS for Http {
    fn request(&self, url: &Url, state: Arc<LockLatch<Response>>) {
        let xhr = Rc::new(RefCell::new(Xhr::new(state)));
        let clone = xhr.clone();

        xhr.borrow_mut().on_load = Some(Closure::wrap(Box::new(move || {
            let xhr = clone.borrow();
            let rsp = xhr.inner.response().unwrap();
            let array = Uint8Array::new(&rsp);

            // FIXME: https://github.com/rustwasm/wasm-bindgen/issues/811
            let mut bytes = Vec::new();
            array.for_each(&mut |v, _, _| bytes.push(v));

            xhr.state.set(Ok(bytes.into_boxed_slice()));
        })));

        {
            let xhr = xhr.borrow();

            if let Some(closure) = xhr.on_load.as_ref() {
                (xhr.inner.as_ref() as &web_sys::EventTarget)
                    .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
                    .unwrap();
            }

            let ty = XmlHttpRequestResponseType::Arraybuffer;
            xhr.inner.set_response_type(ty);

            xhr.inner.open_with_async("Get", url, true).unwrap();
            xhr.inner.send().unwrap();
        }
    }
}

struct Xhr {
    inner: XmlHttpRequest,
    on_load: Option<Closure<FnMut()>>,
    state: Arc<LockLatch<Response>>,
}

impl Xhr {
    pub fn new(state: Arc<LockLatch<Response>>) -> Self {
        Xhr {
            inner: XmlHttpRequest::new().unwrap(),
            state: state,
            on_load: None,
        }
    }
}
