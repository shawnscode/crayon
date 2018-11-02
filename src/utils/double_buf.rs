use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct DoubleBuf<T> {
    wbuf: RwLock<T>,
    rbuf: RwLock<T>,
}

impl<T: Default> Default for DoubleBuf<T> {
    fn default() -> Self {
        DoubleBuf {
            wbuf: RwLock::new(Default::default()),
            rbuf: RwLock::new(Default::default()),
        }
    }
}

impl<T> DoubleBuf<T> {
    #[inline]
    pub fn new(w: T, r: T) -> Self {
        DoubleBuf {
            wbuf: RwLock::new(w),
            rbuf: RwLock::new(r),
        }
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.wbuf.write().unwrap()
    }

    #[inline]
    pub fn write_back_buf(&self) -> RwLockWriteGuard<T> {
        self.rbuf.write().unwrap()
    }

    #[inline]
    pub fn read_back_buf(&self) -> RwLockReadGuard<T> {
        self.rbuf.read().unwrap()
    }

    #[inline]
    pub fn swap(&self) {
        let mut wbuf = self.wbuf.write().unwrap();
        let mut rbuf = self.rbuf.write().unwrap();
        ::std::mem::swap::<T>(&mut wbuf, &mut rbuf);
    }
}
