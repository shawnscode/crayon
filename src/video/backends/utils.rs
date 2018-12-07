use std::borrow::Borrow;

use crate::utils::handle::Handle;

#[derive(Debug)]
pub struct DataVec<T>
where
    T: Sized + Clone,
{
    pub buf: Vec<Option<T>>,
    pub versions: Vec<u32>,
}

impl<T> DataVec<T>
where
    T: Sized + Clone,
{
    pub fn new() -> Self {
        DataVec {
            buf: Vec::new(),
            versions: Vec::new(),
        }
    }

    pub fn get<H>(&self, handle: H) -> Option<&T>
    where
        H: Borrow<Handle>,
    {
        let index = handle.borrow().index() as usize;
        if let Some(&v) = self.versions.get(index) {
            if v == handle.borrow().version() {
                return self.buf[index].as_ref();
            }
        }

        None
    }

    pub fn create<H>(&mut self, handle: H, value: T)
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        let index = handle.index() as usize;

        if self.buf.len() <= index {
            self.buf.resize(index + 1, None);
            self.versions.resize(index + 1, 1);
        }

        self.buf[index] = Some(value);
        self.versions[index] = handle.version();
    }

    pub fn free<H>(&mut self, handle: H) -> Option<T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.buf.len() <= handle.index() as usize {
            None
        } else {
            let mut value = None;
            ::std::mem::swap(&mut value, &mut self.buf[handle.index() as usize]);
            value
        }
    }
}
