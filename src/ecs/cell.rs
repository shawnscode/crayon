//! A thread-safe `RefCell`.

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Wraps a borrowed reference to a value in a `RefCell<T>` box. A wrapper
/// type for an immutably borrowed value from a `RefCell<T>`.
#[derive(Debug)]
pub struct Ref<'a, T: 'a> {
    flag: &'a AtomicUsize,
    value: &'a T,
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.flag.fetch_sub(1, Ordering::Release);
    }
}

/// A wrapper type for a mutably borrowed value from a `RefCell<T>`.
#[derive(Debug)]
pub struct RefMut<'a, T: 'a> {
    flag: &'a AtomicUsize,
    value: &'a mut T,
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.flag.store(0, Ordering::Release)
    }
}

/// A thread-safe `RefCell`.
#[derive(Debug)]
pub struct RefCell<T> {
    flag: AtomicUsize,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for RefCell<T>
where
    T: Sync,
{
}

impl<T> RefCell<T> {
    /// Creates a new `RefCell` containing value.
    pub fn new(val: T) -> Self {
        RefCell {
            flag: AtomicUsize::new(0),
            inner: UnsafeCell::new(val),
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned Ref exits scope. Multiple immutable borrows
    /// can be taken out at the same time.
    ///
    /// #Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow(&self) -> Ref<T> {
        loop {
            let val = self.flag.load(Ordering::Acquire);
            assert!(val != !0, "Already borrowed mutably");

            if self.flag.compare_and_swap(val, val + 1, Ordering::AcqRel) == val {
                return Ref {
                    flag: &self.flag,
                    value: unsafe { &*self.inner.get() },
                };
            }
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned RefMut exits scope. The value cannot be
    /// borrowed while this borrow is active.
    ///
    /// Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut(&self) -> RefMut<T> {
        let val = self.flag.compare_and_swap(0, !0, Ordering::AcqRel);
        assert!(val == 0, "Already borrowed");

        RefMut {
            flag: &self.flag,
            value: unsafe { &mut *self.inner.get() },
        }
    }
}
