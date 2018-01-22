use application;
use ecs;

use scene::{camera, light, node, transform};
use scene::pbr::renderer;
use scene::errors::*;


#[derive(Debug, Clone, Copy)]
pub enum PbrElement {
    None,
    Light(light::Light),
    Camera(camera::Camera),
    Mesh(renderer::PbrMesh),
}

impl ecs::Component for PbrElement {
    type Arena = ecs::VecArena<PbrElement>;
}

pub struct PbrScene {
    world: ecs::World,
}

impl PbrScene {
    pub fn new() -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<node::Node>();
        world.register::<transform::Transform>();
        world.register::<PbrElement>();
        Ok(PbrScene { world: world })
    }

    #[inline(always)]
    pub fn arena<T>(&self) -> ecs::Fetch<T>
    where
        T: ecs::Component,
    {
        self.world.arena::<T>()
    }

    #[inline(always)]
    pub fn arena_mut<T>(&self) -> ecs::FetchMut<T>
    where
        T: ecs::Component,
    {
        self.world.arena_mut::<T>()
    }

    #[inline(always)]
    pub fn free(&mut self, handle: ecs::Entity) -> Result<()> {
        node::Node::remove_from_parent(&mut self.arena_mut::<node::Node>(), handle)?;
        self.world.free(handle);
        Ok(())
    }

    #[inline(always)]
    pub fn create_camera(&mut self, camera: camera::Camera) -> ecs::Entity {
        self.world
            .build()
            .with_default::<node::Node>()
            .with_default::<transform::Transform>()
            .with(PbrElement::Camera(camera))
            .finish()
    }

    #[inline(always)]
    pub fn create_light(&mut self, light: light::Light) -> ecs::Entity {
        self.world
            .build()
            .with_default::<node::Node>()
            .with_default::<transform::Transform>()
            .with(PbrElement::Light(light))
            .finish()
    }

    pub fn update(&mut self, _: &application::Context) -> Result<()> {
        Ok(())
    }

    pub fn render(&mut self, ctx: &application::Context) -> Result<()> {
        // self.camera.run_at(&self.world);

        // if let Some(camera) = self.camera.main {
        //     self.render.run_at(&self.world);
        // }

        Ok(())
    }
}

// type RenderData<'a> = (
//     ecs::Fetch<'a, transform::Transform>,
//     ecs::Fetch<'a, node::Node>,
//     ecs::Fetch<'a, camera::Camera>,
// );

// struct RenderSystem {}

// impl<'a> ecs::System<'a> for RenderSystem {
//     type ViewWith = RenderData<'a>;

//     fn run(&mut self, view: ecs::View, _: Self::ViewWith) {
//         for v in view {
//             println!("draw {:?}", v);
//         }
//     }
// }

// struct CameraSystem {
//     main: Option<ecs::Entity>,
// }

// impl<'a> ecs::System<'a> for CameraSystem {
//     type ViewWith = RenderData<'a>;

//     fn run(&mut self, view: ecs::View, _: Self::ViewWith) {
//         if self.main.is_none() {
//             for v in view {
//                 self.main = Some(v);
//                 return;
//             }
//         }
//     }
// }
