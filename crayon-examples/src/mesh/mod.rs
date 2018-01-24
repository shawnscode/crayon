use crayon::prelude::*;
use crayon_imgui::prelude::*;
use crayon::scene::material::MaterialHandle;
use utils::*;

struct Window {
    surface: SurfaceHandle,
    scene: Scene,
    console: ConsoleCanvas,

    material: MaterialHandle,
    camera: Entity,
    room: Entity,
    rotation: math::Vector3<f32>,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
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
            let c = Camera::perspective(math::Deg(60.0), 6.4 / 4.8, 0.1, 1000.0);
            scene.create_node(c)
        };

        let (room, mat_block) = Window::create_room(&mut scene, &video)?;
        Window::create_lits(&mut scene, &video)?;

        let light = scene.create_node(Light::default());

        {
            let tree = scene.arena::<Node>();
            let mut transforms = scene.arena_mut::<Transform>();

            let zero = [0.0, 0.0, 0.0];
            let up = [0.0, 1.0, 0.0];
            Transform::set_world_position(&tree, &mut transforms, camera, [0.0, 0.0, -500.0])?;
            Transform::look_at(&tree, &mut transforms, camera, zero, up)?;

            Transform::set_world_position(&tree, &mut transforms, light, [0.0, 500.0, -500.0])?;
            Transform::look_at(&tree, &mut transforms, light, zero, up)?;
        }

        Ok(Window {
            console: ConsoleCanvas::new(1, ctx)?,
            surface: surface,
            scene: scene,
            camera: camera,
            room: room,
            rotation: math::Vector3::new(0.0, 0.0, 0.0),
            material: mat_block,
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
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
    ) -> errors::Result<(Entity, MaterialHandle)> {
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
        Transform::set_world_scale(&tree, &mut transforms, room, 0.6)?;
        Ok((room, mat_block))
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let ambient = &mut self.ambient;
        let diffuse = &mut self.diffuse;
        let specular = &mut self.specular;

        let capture = {
            let canvas = self.console.render(&ctx);
            canvas
                .window(im_str!("Materials"))
                .movable(false)
                .resizable(false)
                .position((0.0, 70.0), ImGuiCond::FirstUseEver)
                .size((250.0, 150.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    canvas
                        .slider_float3(im_str!("u_Ambient"), ambient, 0.0, 1.0)
                        .build();
                    canvas
                        .slider_float3(im_str!("u_Diffuse"), diffuse, 0.0, 1.0)
                        .build();
                    canvas
                        .slider_float3(im_str!("u_Specular"), specular, 0.0, 1.0)
                        .build();
                });

            canvas.want_capture_mouse()
        };

        if !capture {
            let input = ctx.shared::<InputSystem>();
            match input.finger_pan() {
                input::GesturePan::Move {
                    start_position: _,
                    position: _,
                    movement,
                } => {
                    self.rotation.y -= movement.y;
                    self.rotation.x -= movement.x;
                    let euler = math::Euler::new(
                        math::Deg(self.rotation.y),
                        math::Deg(self.rotation.x),
                        math::Deg(self.rotation.z),
                    );
                    unsafe {
                        let mut transforms = self.scene.arena_mut::<Transform>();
                        transforms.get_unchecked_mut(self.room).set_rotation(euler);
                    }
                }
                _ => {}
            };
        }

        self.scene
            .update_material_uniform(self.material, "u_Ambient", *ambient)?;
        self.scene
            .update_material_uniform(self.material, "u_Diffuse", *diffuse)?;
        self.scene
            .update_material_uniform(self.material, "u_Specular", *specular)?;
        self.scene.render(self.surface, self.camera)?;
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> errors::Result<()> {
        self.console.update(info);
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
