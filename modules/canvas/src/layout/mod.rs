use crayon::{ecs, math, application};
use crayon::ecs::VecArena;

use prelude::Element;
use errors::*;

#[derive(Debug, Copy, Clone)]
pub enum LayoutController {
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub position: math::Vector2<f32>,

    /// The design size of this widget.
    pub size: math::Vector2<f32>,
    pub(crate) fixed_size: Option<math::Vector2<f32>>,

    /// The normalized position in the parent widget that the lower left corner is anchored to.
    pub anchor_min: math::Vector2<f32>,
    /// The normalized position in the parent widget that the upper right corner is anchored to.
    pub anchor_max: math::Vector2<f32>,
    ///	The normalized position in this widget that it rotates around.
    pub pivot: math::Vector2<f32>,
    /// The layout controller of this item.
    pub layout: Option<LayoutController>,
}

declare_component!(Layout, VecArena);

impl Default for Layout {
    fn default() -> Self {
        Layout {
            position: math::Vector2::new(0.0, 0.0),
            size: math::Vector2::new(0.0, 0.0),
            fixed_size: None,
            anchor_min: math::Vector2::new(0.5, 0.5),
            anchor_max: math::Vector2::new(0.5, 0.5),
            pivot: math::Vector2::new(0.5, 0.5),
            layout: None,
        }
    }
}

impl Layout {
    pub unsafe fn perform(ctx: &application::Context,
                          elements: &ecs::Arena<Element>,
                          layouts: &mut ecs::ArenaMut<Layout>,
                          parent: ecs::Entity,
                          children: &[ecs::Entity])
                          -> Result<()> {
        let ctrl = layouts.get_unchecked_mut(parent).layout;

        match ctrl {
            None => {
                for v in children {
                    let e = elements.get_unchecked(*v);
                    let l = layouts.get_unchecked_mut(*v);

                    let prefered_size = e.prefered_size(ctx).unwrap_or(l.size);
                    l.size = l.fixed_size.unwrap_or(prefered_size);
                }
            }

            Some(_) => {}
        }

        Ok(())
    }
}
