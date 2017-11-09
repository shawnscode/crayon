use std::sync::Arc;
use std::collections::VecDeque;

use crayon::{ecs, application, utils};
use crayon::ecs::Arena;

use errors::*;
use node::Node;
use element::Element;
use renderer::CanvasRenderer;
use layout::Layout;
use assets::{Font, FontHandle};

pub struct CanvasSystem {
    world: ecs::World,
    entities: ecs::Entity,
    fonts: utils::ObjectPool<Arc<Font>>,
    renderer: CanvasRenderer,

    // design_resolution: (u32, u32),
    // design_dpi: u32,
}

impl CanvasSystem {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<Node>();
        world.register::<Element>();
        world.register::<Layout>();

        let renderer = CanvasRenderer::new(ctx)?;
        let root = world
            .build()
            .with_default::<Node>()
            .with_default::<Layout>()
            .with_default::<Element>()
            .finish();

        Ok(CanvasSystem {
               entities: root,
               world: world,
               fonts: utils::ObjectPool::new(),
               renderer: renderer,
           })
    }

    pub fn world(&self) -> &ecs::World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut ecs::World {
        &mut self.world
    }

    pub fn create(&mut self) -> ecs::Entity {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Layout>()
            .with_default::<Element>()
            .finish()
    }

    pub fn create_text(&mut self) -> ecs::Entity {
        use element::text;
        self.world
            .build()
            .with_default::<Node>()
            .with::<Element>(Element::Text(text::Text::default()))
            .finish()
    }

    ///
    pub fn perform_layout(&mut self, ctx: &application::Context) -> Result<()> {
        let mut children = Vec::new();

        {
            let (view, arena) = self.world.view_with::<Node>();
            for node in view {
                unsafe {
                    if arena.get_unchecked(node).is_root() {
                        children.push(node);
                    }
                }
            }
        }

        let mut nodes = self.world.arena::<Node>();
        let mut elements = self.world.arena::<Element>();
        let mut layouts = self.world.arena_mut::<Layout>();

        unsafe {
            Layout::perform(ctx, &elements, &mut layouts, self.entities, &children)?;

            for v in children {
                Self::perform_layout_recursive(ctx, &nodes, &elements, &mut layouts, v)?;
            }
        }

        Ok(())
    }

    fn perform_layout_recursive(ctx: &application::Context,
                                nodes: &ecs::Arena<Node>,
                                elements: &ecs::Arena<Element>,
                                layouts: &mut ecs::ArenaMut<Layout>,
                                ent: ecs::Entity)
                                -> Result<()> {
        let children: Vec<_> = Node::children(nodes, ent).collect();

        unsafe {
            Layout::perform(ctx, elements, layouts, ent, &children)?;
            for v in children {
                Self::perform_layout_recursive(ctx, nodes, elements, layouts, v)?;
            }
        }

        Ok(())
    }

    /// Draw the whole scene.
    pub fn draw(&mut self, ctx: &application::Context) -> Result<()> {
        let mut children = Vec::new();

        {
            let (view, arena) = self.world.view_with::<Node>();
            for node in view {
                unsafe {
                    if arena.get_unchecked(node).is_root() {
                        children.push(node);
                    }
                }
            }
        }

        let nodes = self.world.arena::<Node>();
        let elements = self.world.arena::<Element>();

        unsafe {
            for v in children {
                elements.get_unchecked(v).draw(ctx);
                Self::draw_recursive(ctx, &nodes, &elements, v)?;
            }
        }

        self.renderer.flush(ctx)?;
        Ok(())
    }

    unsafe fn draw_recursive(ctx: &application::Context,
                             nodes: &ecs::Arena<Node>,
                             elements: &ecs::Arena<Element>,
                             ent: ecs::Entity)
                             -> Result<()> {
        for v in Node::children(nodes, ent) {
            elements.get_unchecked(v).draw(ctx);
            Self::draw_recursive(ctx, nodes, elements, v)?;
        }

        Ok(())
    }

    #[inline]
    pub fn font(&self, handle: FontHandle) -> Option<&Arc<Font>> {
        self.fonts.get(handle)
    }
}