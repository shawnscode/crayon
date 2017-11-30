use std::borrow::Borrow;
use crayon::{ecs, application, math, utils};
use crayon::ecs::{Arena, ArenaMut};

use errors::*;
use node::Node;
use element::Element;
use renderer::CanvasRenderer;
use layout::Layout;
use assets::FontSystem;

pub struct CanvasSystem {
    world: ecs::World,
    screen: ecs::Entity,
    fonts: FontSystem,
    renderer: CanvasRenderer,
    design_resolution: (f32, f32),
    dpi_factor: f32,
}

impl CanvasSystem {
    pub fn new(ctx: &application::Context,
               design_resolution: (f32, f32),
               dpi_factor: f32)
               -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<Node>();
        world.register::<Element>();
        world.register::<Layout>();

        let fonts = FontSystem::new(ctx);
        let renderer = CanvasRenderer::new(ctx)?;

        let mut layout = Layout::default();
        layout.anchor_min = math::Vector2::new(0.0, 0.0);
        layout.anchor_max = math::Vector2::new(1.0, 1.0);
        layout.pivot = math::Vector2::new(0.0, 0.0);
        layout.size = design_resolution.into();
        layout.fixed_size = Some(design_resolution.into());

        let screen = world
            .build()
            .with_default::<Node>()
            .with_default::<Element>()
            .with::<Layout>(layout)
            .finish();

        Ok(CanvasSystem {
               world: world,
               screen: screen,
               renderer: renderer,
               fonts: fonts,
               design_resolution: design_resolution,
               dpi_factor: dpi_factor,
           })
    }

    /// Sets the current hdpi-factor.
    pub fn set_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    /// Creates a new and empty node. It has no parent in the hierary, and will be
    /// treated as root initially.
    pub fn create(&mut self) -> ecs::Entity {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Layout>()
            .with_default::<Element>()
            .finish()
    }

    /// Sets the element component for specified code.
    pub fn set_element(&mut self, node: ecs::Entity, element: Element) {
        unsafe {
            *self.world.arena_mut::<Element>().get_unchecked_mut(node) = element;
        }
    }

    /// Sets the layout component for specified node.
    pub fn set_layout(&mut self, node: ecs::Entity, layout: Layout) {
        unsafe {
            *self.world.arena_mut::<Layout>().get_unchecked_mut(node) = layout;
        }
    }

    /// Advances the `CanvasSystem` with essential updates.
    pub fn advance(&mut self) -> Result<()> {
        self.fonts.set_dpi_factor(self.dpi_factor);
        self.fonts.advance();
        Ok(())
    }

    /// Layouts all the active nodes. This should be called every frames before drawing.
    pub fn perform_layout(&mut self, _: &application::Context) -> Result<()> {
        let children = self.collect_root_nodes();
        let fonts = &mut self.fonts;
        let nodes = self.world.arena::<Node>();
        let elements = self.world.arena::<Element>();
        let mut layouts = self.world.arena_mut::<Layout>();

        unsafe {
            Layout::perform(fonts, &elements, &mut layouts, self.screen, &children)?;

            for v in children {
                Self::perform_layout_recursive(fonts, &nodes, &elements, &mut layouts, v)?;
            }
        }

        Ok(())
    }

    unsafe fn perform_layout_recursive(fonts: &mut FontSystem,
                                       nodes: &ecs::Arena<Node>,
                                       elements: &ecs::Arena<Element>,
                                       layouts: &mut ecs::ArenaMut<Layout>,
                                       node: ecs::Entity)
                                       -> Result<()> {
        Layout::perform(fonts, elements, layouts, node, Node::children(nodes, node))?;

        for v in Node::children(nodes, node) {
            Self::perform_layout_recursive(fonts, nodes, elements, layouts, v)?;
        }

        Ok(())
    }

    /// Draws the whole scene.
    pub fn draw(&mut self, _: &application::Context) -> Result<()> {
        let hsize = self.design_resolution.0;
        let vsize = self.design_resolution.1;
        let transform: math::Matrix4<f32> = math::ortho(0.0, hsize, 0.0, vsize, 0.0, 1.0).into();
        let children = self.collect_root_nodes();

        unsafe {
            Self::draw_recursive(&mut self.renderer,
                                 &mut self.fonts,
                                 &self.world.arena::<Node>(),
                                 &self.world.arena::<Element>(),
                                 &mut self.world.arena_mut::<Layout>(),
                                 transform,
                                 &children)?;
        }

        self.renderer.flush()?;
        Ok(())
    }

    unsafe fn draw_recursive<T, U>(renderer: &mut CanvasRenderer,
                                   fonts: &mut FontSystem,
                                   nodes: &ecs::Arena<Node>,
                                   elements: &ecs::Arena<Element>,
                                   layouts: &ecs::ArenaMut<Layout>,
                                   transform: math::Matrix4<f32>,
                                   children: T)
                                   -> Result<()>
        where T: IntoIterator<Item = U>,
              U: Borrow<utils::Handle>
    {
        for v in children {
            let node = *v.borrow();
            let l = layouts.get_unchecked(node);
            let t = transform * l.matrix();

            renderer.set_matrix(t);
            elements.get_unchecked(node).draw(renderer, fonts, l.size)?;

            let c = Node::children(nodes, node);
            Self::draw_recursive(renderer, fonts, nodes, elements, layouts, t, c)?;
        }

        Ok(())
    }

    fn collect_root_nodes(&self) -> Vec<ecs::Entity> {
        let mut children = Vec::new();

        unsafe {
            let (view, arena) = self.world.view_with::<Node>();
            for node in view {
                if arena.get_unchecked(node).is_root() && node != self.screen {
                    children.push(node);
                }
            }
        }

        children
    }
}