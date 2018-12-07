use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::errors::Result;
use crate::sched::prelude::LatchProbe;

/// The Launcher will starts the user defined LifecycleListener after the resources probe has been
/// set.
pub struct Launcher<T1: LatchProbe, T2: LifecycleListener + 'static> {
    resources: T1,
    state: LaunchState<T1, T2>,
}

type Wrapper<T1, T2> = Box<dyn for<'r> FnMut(&'r T1) -> Result<T2> + Send>;

impl<T1: LatchProbe, T2: LifecycleListener + 'static> Launcher<T1, T2> {
    pub fn new<F: for<'r> FnOnce(&'r T1) -> Result<T2> + Send + 'static>(
        resources: T1,
        closure: F,
    ) -> Self {
        let mut v = Some(closure);
        let wrapper: Wrapper<T1, T2> = Box::new(move |r| {
            let mut w = None;
            std::mem::swap(&mut v, &mut w);
            w.unwrap()(r)
        });

        Launcher {
            resources,
            state: LaunchState::NotReady(wrapper),
        }
    }
}

enum LaunchState<T1: LatchProbe, T2: LifecycleListener + 'static> {
    NotReady(Wrapper<T1, T2>),
    Ok(LifecycleListenerHandle),
}

impl<T1: LatchProbe, T2: LifecycleListener + 'static> Drop for Launcher<T1, T2> {
    fn drop(&mut self) {
        if let LaunchState::Ok(lis) = self.state {
            crate::application::detach(lis);
        }
    }
}

impl<T1: LatchProbe, T2: LifecycleListener + 'static> LifecycleListener for Launcher<T1, T2> {
    fn on_update(&mut self) -> Result<()> {
        let lis = match self.state {
            LaunchState::NotReady(ref mut closure) => {
                if self.resources.is_set() {
                    let v = closure(&self.resources)?;
                    Some(crate::application::attach(v))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(lis) = lis {
            self.state = LaunchState::Ok(lis);
        }

        Ok(())
    }
}
