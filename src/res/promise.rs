use std::sync::{Condvar, Mutex};

use errors::*;

enum PromiseState {
    NotReady,
    Ok(Result<()>),
}

pub struct Promise {
    m: Mutex<PromiseState>,
    v: Condvar,
}

impl Promise {
    #[inline]
    pub fn new() -> Self {
        Promise {
            m: Mutex::new(PromiseState::NotReady),
            v: Condvar::new(),
        }
    }

    #[inline]
    pub(crate) fn set(&self, v: Result<()>) {
        {
            let mut guard = self.m.lock().unwrap();
            *guard = PromiseState::Ok(v);
        }

        self.v.notify_all();
    }

    #[inline]
    pub fn take(&self) -> Result<()> {
        let mut guard = self.m.lock().unwrap();
        if let PromiseState::Ok(v) = ::std::mem::replace(&mut *guard, PromiseState::Ok(Ok(()))) {
            v
        } else {
            unreachable!();
        }
    }

    fn is_set(&self) -> bool {
        let guard = self.m.lock().unwrap();
        if let PromiseState::NotReady = *guard {
            false
        } else {
            true
        }
    }

    fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while let PromiseState::NotReady = *guard {
            guard = self.v.wait(guard).unwrap();
        }
    }
}
