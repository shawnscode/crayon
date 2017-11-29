use std::borrow::Borrow;

use crayon::{ecs, math, utils};
use crayon::ecs::VecArena;
use crayon::math::Transform;

use assets::FontSystem;
use prelude::Element;
use errors::*;

#[derive(Debug, Copy, Clone)]
pub enum LayoutController {
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub decomposed: math::Decomposed<math::Vector3<f32>, math::Quaternion<f32>>,

    /// The calculated size of this widget after performing layout.
    pub(crate) size: math::Vector2<f32>,
    /// The design size of this widget.
    pub fixed_size: Option<math::Vector2<f32>>,

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
            decomposed: math::Decomposed::one(),
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
    pub fn set_position<T>(&mut self, position: T)
        where T: Into<math::Vector2<f32>>
    {
        let p = position.into();
        self.decomposed.disp = math::Vector3::new(p.x, p.y, 0.0);
    }

    pub fn set_pivot<T>(&mut self, pivot: T)
        where T: Into<math::Vector2<f32>>
    {
        self.pivot = pivot.into();
        self.pivot.x = self.pivot.x.min(1.0).max(0.0);
        self.pivot.y = self.pivot.y.min(1.0).max(0.0);
    }

    pub fn matrix(&self) -> math::Matrix4<f32> {
        let mut decomposed = self.decomposed;
        decomposed.disp.x -= self.pivot.x * self.size.x;
        decomposed.disp.y += (1.0 - self.pivot.y) * self.size.y;
        math::Matrix4::from(decomposed)
    }
}

impl Layout {
    pub unsafe fn perform<T, U>(fonts: &mut FontSystem,
                                elements: &ecs::Arena<Element>,
                                layouts: &mut ecs::ArenaMut<Layout>,
                                parent: ecs::Entity,
                                children: T)
                                -> Result<()>
        where T: IntoIterator<Item = U>,
              U: Borrow<utils::Handle>
    {
        let ctrl = layouts.get_unchecked_mut(parent).layout;

        match ctrl {
            None => {
                for v in children {
                    let e = elements.get_unchecked(*v.borrow());
                    let l = layouts.get_unchecked_mut(*v.borrow());

                    let prefered_size = e.prefered_size(fonts).unwrap_or(l.size);
                    l.size = l.fixed_size.unwrap_or(prefered_size);
                }
            }

            Some(_) => {}
        }

        Ok(())
    }
}
