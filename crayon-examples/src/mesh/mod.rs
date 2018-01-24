use crayon::prelude::*;
use utils::*;

struct Window {
    surface: SurfaceHandle,
    scene: Scene,

    camera: Entity,
    room: Entity,
    rotation: math::Vector3<f32>,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        engine.resource.mount("std", DirectoryFS::new("assets")?)?;
        engine.input.set_touch_emulation(true);

        let ctx = engine.context();
        let video = ctx.shared::<GraphicsSystem>().clone();

        // Create the view state.
        let setup = graphics::SurfaceSetup::default();
        let surface = video.create_surface(setup)?;

        // Create scene.
        let mut scene = Scene::new(&ctx)?;

        let camera = {
            let c = Camera::perspective(math::Deg(60.0), 4.8 / 3.2, 0.1, 1000.0);
            scene.create_node(c)
        };

        let room = Window::create_room(&mut scene, &video)?;
        Window::create_lits(&mut scene, &video)?;

        let light = scene.create_node(Light::default());

        {
            let tree = scene.arena::<Node>();
            let mut transforms = scene.arena_mut::<Transform>();

            let zero = [0.0, 0.0, 0.0];
            let up = [0.0, 1.0, 0.0];
            Transform::set_world_position(&tree, &mut transforms, camera, [0.0, 0.0, -500.0])?;
            Transform::look_at(&tree, &mut transforms, camera, zero, up)?;

            Transform::set_world_position(&tree, &mut transforms, light, [0.0, 0.0, -500.0])?;
            Transform::look_at(&tree, &mut transforms, light, zero, up)?;
        }

        Ok(Window {
            surface: surface,
            scene: scene,
            camera: camera,
            room: room,
            rotation: math::Vector3::new(0.0, 0.0, 0.0),
        })
    }

    fn create_lits(scene: &mut Scene, video: &GraphicsSystemShared) -> errors::Result<[Entity; 4]> {
        // Create shader state.
        let shader = scene::factory::shader::color(&video)?;
        let mesh = scene::factory::mesh::cube(&video)?;

        let mut lits = [Entity::nil(); 4];
        let colors = [Color::red(), Color::blue(), Color::green(), Color::cyan()];
        let positions = [
            [100.0, 0.0, 0.0],
            [-100.0, 0.0, 0.0],
            [0.0, 100.0, 0.0],
            [0.0, -100.0, 0.0],
        ];

        for i in 0..4 {
            let node = scene.create_node(());

            let lit = scene.create_node(Light {
                enable: true,
                color: colors[i],
                intensity: 1.0,
                source: LightSource::Point {
                    radius: 100.0,
                    smoothness: 0.001,
                },
            });

            let mat = scene.create_material(shader)?;
            let color: [f32; 4] = colors[i].into();
            scene.update_material_uniform(mat, "u_Color", color)?;

            let cube = scene.create_node(MeshRenderer {
                mesh: mesh,
                index: MeshIndex::All,
                material: mat,
            });

            unsafe {
                let mut tree = scene.arena_mut::<Node>();
                let mut transforms = scene.arena_mut::<Transform>();
                Node::set_parent(&mut tree, lit, node)?;
                Node::set_parent(&mut tree, cube, lit)?;
                transforms.get_unchecked_mut(cube).set_scale(20.0);
                transforms.get_unchecked_mut(lit).set_position(positions[i]);
            }

            lits[i] = node;
        }

        Ok(lits)
    }

    fn create_room(
        scene: &mut Scene,
        video: &graphics::GraphicsSystemShared,
    ) -> errors::Result<Entity> {
        // Create shader state.
        let shader = scene::factory::shader::phong(&video)?;

        let setup = graphics::MeshSetup::default();
        let mesh = video
            .create_mesh_from::<OBJParser>(Location::shared(0, "/std/cornell_box.obj"), setup)?;

        let mat_wall = scene.create_material(shader)?;
        scene.update_material_uniform(mat_wall, "u_Ambient", [1.0, 1.0, 1.0])?;
        scene.update_material_uniform(mat_wall, "u_Diffuse", [1.0, 1.0, 1.0])?;
        scene.update_material_uniform(mat_wall, "u_Specular", [0.0, 0.0, 0.0])?;
        scene.update_material_uniform(mat_wall, "u_Shininess", 0.0)?;

        let mat_block = scene.create_material(shader)?;
        scene.update_material_uniform(mat_block, "u_Ambient", [1.0, 1.0, 1.0])?;
        scene.update_material_uniform(mat_block, "u_Diffuse", [1.0, 1.0, 1.0])?;
        scene.update_material_uniform(mat_block, "u_Specular", [1.0, 1.0, 1.0])?;
        scene.update_material_uniform(mat_block, "u_Shininess", 0.5)?;

        let room = scene.create_node(());
        let anchor = [-278.0, -274.0, 280.0];

        for i in 0..6 {
            let wall = scene.create_node(MeshRenderer {
                mesh: mesh,
                index: MeshIndex::SubMesh(i),
                material: mat_wall,
            });

            let mut tree = scene.arena_mut::<Node>();
            let mut transforms = scene.arena_mut::<Transform>();
            Node::set_parent(&mut tree, wall, room)?;
            Transform::set_world_position(&tree, &mut transforms, wall, anchor)?;
        }

        for i in 6..8 {
            let block = scene.create_node(MeshRenderer {
                mesh: mesh,
                index: MeshIndex::SubMesh(i),
                material: mat_block,
            });

            let mut tree = scene.arena_mut::<Node>();
            let mut transforms = scene.arena_mut::<Transform>();
            Node::set_parent(&mut tree, block, room)?;
            Transform::set_world_position(&tree, &mut transforms, block, anchor)?;
        }

        let tree = scene.arena::<Node>();
        let mut transforms = scene.arena_mut::<Transform>();
        Transform::set_world_scale(&tree, &mut transforms, room, 0.5)?;
        Ok(room)
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let input = ctx.shared::<InputSystem>();
        unsafe {
            let mut transforms = self.scene.arena_mut::<Transform>();
            match input.finger_pan() {
                input::GesturePan::Move {
                    start_position,
                    position,
                    movement,
                } => {
                    self.rotation.y -= movement.y;
                    self.rotation.x -= movement.x;
                    let euler = math::Euler::new(
                        math::Deg(self.rotation.y),
                        math::Deg(self.rotation.x),
                        math::Deg(self.rotation.z),
                    );
                    transforms.get_unchecked_mut(self.room).set_rotation(euler);
                }
                _ => {}
            };
        }

        self.scene.render(self.surface, self.camera)?;
        Ok(())
    }
}

pub fn main(title: String, _: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 640;
    settings.window.height = 480;
    settings.window.title = title;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
