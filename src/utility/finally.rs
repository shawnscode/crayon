pub fn finally<A, F>(arg: A, func: F) -> FinallyGuard<A, F>
    where F: FnMut(&mut A)
{
    FinallyGuard {
        arg: arg,
        func: func,
    }
}

pub struct FinallyGuard<A, F>
    where F: FnMut(&mut A)
{
    arg: A,
    func: F,
}

impl<A, F> FinallyGuard<A, F>
    where F: FnMut(&mut A)
{
    pub fn forget(self) {
        ::std::mem::forget(self);
    }
}

impl<A, F> Drop for FinallyGuard<A, F>
    where F: FnMut(&mut A)
{
    fn drop(&mut self) {
        (self.func)(&mut self.arg)
    }
}