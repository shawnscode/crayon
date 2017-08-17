pub fn finally_with<A, F>(arg: A, func: F) -> FinallyGuardWith<A, F>
    where F: FnMut(&mut A)
{
    FinallyGuardWith {
        arg: arg,
        func: func,
    }
}

pub fn finally<F>(func: F) -> FinallyGuard<F>
    where F: FnMut()
{
    FinallyGuard { func: func }
}

pub struct FinallyGuardWith<A, F>
    where F: FnMut(&mut A)
{
    arg: A,
    func: F,
}

impl<A, F> FinallyGuardWith<A, F>
    where F: FnMut(&mut A)
{
    pub fn forget(self) {
        ::std::mem::forget(self);
    }
}

impl<A, F> Drop for FinallyGuardWith<A, F>
    where F: FnMut(&mut A)
{
    fn drop(&mut self) {
        (self.func)(&mut self.arg)
    }
}

pub struct FinallyGuard<F>
    where F: FnMut()
{
    func: F,
}

impl<F> FinallyGuard<F>
    where F: FnMut()
{
    pub fn forget(self) {
        ::std::mem::forget(self);
    }
}

impl<F> Drop for FinallyGuard<F>
    where F: FnMut()
{
    fn drop(&mut self) {
        (self.func)()
    }
}