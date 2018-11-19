#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceState {
    Ok,
    NotReady,
    Err,
}
