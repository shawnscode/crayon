//! A asynchronous loading request.

use std::sync::Arc;

use sched::prelude::{LatchProbe, LatchWaitProbe, LockLatch};

pub type Response = Result<Box<[u8]>, failure::Error>;

/// A asynchronous loading request. You sould checks the completion status with
/// `poll` method manually. Once the polling returns true, you could fetch the
/// result by `response`.
pub enum Request {
    NotReady(Arc<LockLatch<Response>>),
    Ok(Response),
}

impl Request {
    #[inline]
    pub fn latch() -> Arc<LockLatch<Response>> {
        Arc::new(LockLatch::new())
    }

    #[inline]
    pub fn new(latch: Arc<LockLatch<Response>>) -> Self {
        Request::NotReady(latch)
    }

    #[inline]
    pub fn ok<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        Request::Ok(Ok(bytes.into()))
    }

    #[inline]
    pub fn err<T: Into<failure::Error>>(err: T) -> Self {
        Request::Ok(Err(err.into()))
    }

    /// Blocks the current thread until the request has been resolved.
    #[inline]
    pub fn wait(&mut self) {
        let rsp = match *self {
            Request::NotReady(ref state) => {
                state.wait();
                state.take()
            }
            _ => return,
        };

        *self = Request::Ok(rsp);
    }

    /// Attempt to resolve the request to a final state, and returns true if the
    /// loading result is ready for user.
    #[inline]
    pub fn poll(&mut self) -> bool {
        let rsp = match *self {
            Request::Ok(_) => return true,
            Request::NotReady(ref state) => {
                if !state.is_set() {
                    return false;
                }

                state.take()
            }
        };

        *self = Request::Ok(rsp);
        true
    }

    /// Return the response if exists.
    #[inline]
    pub fn response(&self) -> Option<&Response> {
        match self {
            &Request::Ok(ref rsp) => Some(rsp),
            _ => None,
        }
    }
}

pub struct RequestQueue {
    // FIXME: Use FnOnce instead of Box<Fn> when its stable.
    tasks: Vec<(Request, Box<Fn(&Response)>)>,
}

impl RequestQueue {
    pub fn new() -> Self {
        RequestQueue { tasks: Vec::new() }
    }

    pub fn add<T: Fn(&Response) + 'static>(&mut self, request: Request, func: T) {
        self.tasks.push((request, Box::new(func)));
    }

    pub fn advance(&mut self) {
        // FIXME: Use drain_filter instead of retain and `for` iteration.
        for &mut (ref mut request, ref func) in &mut self.tasks {
            if request.poll() {
                func(request.response().unwrap());
            }
        }

        self.tasks
            .retain(|&(ref request, _)| request.response().is_some());
    }
}
