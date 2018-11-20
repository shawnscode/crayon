use Entity;

/// `Node` is used to store and manipulate the postiion, rotation and scale
/// of the object. Every `Node` can have a parent, which allows you to apply
/// position, rotation and scale hierarchically.
///
/// `Entity` are used to record the tree relationships. Every access requires going
/// through the arena, which can be cumbersome and comes with some runtime overhead.
///
/// But it not only keeps code clean and simple, but also makes `Node` could be
/// send or shared across threads safely. This enables e.g. parallel tree traversals.
#[derive(Default, Debug, Clone, Copy)]
pub struct Node {
    pub parent: Option<Entity>,
    pub next_sib: Option<Entity>,
    pub prev_sib: Option<Entity>,
    pub first_child: Option<Entity>,
}
