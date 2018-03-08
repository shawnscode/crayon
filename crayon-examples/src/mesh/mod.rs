use crayon::prelude::*;
use crayon::graphics::assets::prelude::*;

use crayon_imgui::prelude::*;
use crayon_3d::prelude::*;

use utils::*;
use errors::*;

struct Window {
    scene: Scene,
    console: ConsoleCanvas,

    material: MaterialHandle,
    camera: Entity,
    room: Entity,
    rotation: math::Vector3<f32>,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],

    draw_shadow: bool,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let assets = format!("{0}/assets", env!("CARGO_MANIFEST_DIR"));
        engine.resource.mount("std", DirectoryFS::new(assets)?)?;
        engine.input.set_touch_emulation(true);

        let ctx = engine.context();
        let video = ctx.shared::<GraphicsSystem>().clone();

        // Create scene.
        let mut scene = Scene::new(ctx)?;

        let camera = {
            let c = Camera::perspective(math::Deg(60.0), 6.4 / 4.8, 0.1, 1000.0);
            scene.build().with(c).finish()
        };

        let (room, mat_block) = Window::create_room(&mut scene, &video)?;
        Window::create_lits(&mut scene, &video)?;

        let light = {
            let mut dir = Light::default();
            dir.shadow_caster = true;
            scene.build().with(dir).finish()
        };

        {
            let tree = scene.arena::<Node>();
            let mut transforms = scene.arena_mut::<Transform>();

            let zero = [0.0, 0.0, 0.0];
            let up = [0.0, 1.0, 0.0];
            Transform::set_world_position(&tree, &mut transforms, camera, [0.0, 1.0, -3.0])?;
            // Transform::look_at(&tree, &mut transforms, camera, zero, up)?;

            Transform::set_world_position(&tree, &mut transforms, light, [2.0, 2.0, -2.0])?;
            Transform::look_at(&tree, &mut transforms, light, zero, up)?;
        }

        Ok(Window {
            console: ConsoleCanvas::new(DrawOrder::Max as u64, ctx)?,
            scene: scene,
            camera: camera,
            room: room,
            rotation: math::Vector3::new(0.0, 0.0, 0.0),
            material: mat_block,
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
            draw_shadow: false,
        })
    }

    fn create_lits(scene: &mut Scene, video: &GraphicsSystemShared) -> Result<[Entity; 4]> {
        let shader = factory::pipeline::color(scene)?;
        let mesh = factory::mesh::cube(video)?;

        let mut lits = [Entity::nil(); 4];
        let colors = [Color::red(), Color::blue(), Color::green(), Color::cyan()];
        let positions = [
            [1.0, 0.5, 0.0],
            [-1.0, 0.5, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.5, 0.0],
        ];

        for i in 0..4 {
            let node = scene.build().finish();

            let lit = scene
                .build()
                .with(Light {
                    enable: true,
                    color: colors[i],
                    intensity: 1.0,
                    shadow_caster: false,
                    source: LitSource::Point {
                        radius: 1.0,
                        smoothness: 0.001,
                    },
                })
                .finish();

            let color: [f32; 4] = colors[i].into();
            let mat = scene.create_material(MaterialSetup::new(shader))?;
            scene.update_material(mat, "u_Color", color)?;

            let cube = scene
                .build()
                .with(MeshRenderer {
                    mesh: mesh,
                    materials: vec![mat],
                    shadow_caster: true,
                    shadow_receiver: true,
                    visible: true,
                })
                .finish();

            unsafe {
                let mut tree = scene.arena_mut::<Node>();
                let mut transforms = scene.arena_mut::<Transform>();
                Node::set_parent(&mut tree, lit, node)?;
                Node::set_parent(&mut tree, cube, lit)?;
                transforms.get_unchecked_mut(cube).set_scale(0.1);
                transforms.get_unchecked_mut(lit).set_position(positions[i]);
            }

            lits[i] = node;
        }

        Ok(lits)
    }

    fn create_room(
        scene: &mut Scene,
        video: &GraphicsSystemShared,
    ) -> Result<(Entity, MaterialHandle)> {
        let shader = factory::pipeline::phong(scene)?;

        let mut setup = MeshSetup::default();
        setup.location = Location::shared("/std/cornell_box.obj");
        let mesh = video.create_mesh_from_file::<OBJParser>(setup)?;

        let mat_wall = scene.create_material(MaterialSetup::new(shader))?;
        scene.update_material(mat_wall, "u_Ambient", [1.0, 1.0, 1.0])?;
        scene.update_material(mat_wall, "u_Diffuse", [1.0, 1.0, 1.0])?;
        scene.update_material(mat_wall, "u_Specular", [0.0, 0.0, 0.0])?;
        scene.update_material(mat_wall, "u_Shininess", 0.0)?;

        let mat_block = scene.create_material(MaterialSetup::new(shader))?;
        scene.update_material(mat_block, "u_Ambient", [1.0, 1.0, 1.0])?;
        scene.update_material(mat_block, "u_Diffuse", [1.0, 1.0, 1.0])?;
        scene.update_material(mat_block, "u_Specular", [1.0, 1.0, 1.0])?;
        scene.update_material(mat_block, "u_Shininess", 0.5)?;

        let room = scene
            .build()
            .with(MeshRenderer {
                mesh: mesh,
                materials: vec![
                    mat_wall, mat_wall, mat_wall, mat_wall, mat_wall, mat_block, mat_block,
                    mat_wall,
                ],
                shadow_caster: true,
                shadow_receiver: true,
                visible: true,
            })
            .finish();

        Ok((room, mat_block))
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let ambient = &mut self.ambient;
        let diffuse = &mut self.diffuse;
        let specular = &mut self.specular;

        let capture = {
            let draw_shadow = &mut self.draw_shadow;
            let canvas = self.console.render(ctx);
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

                    if canvas.button(im_str!("Draw Scene"), (100.0, 20.0)) {
                        *draw_shadow = false;
                    }

                    if canvas.button(im_str!("Draw Shadow"), (100.0, 20.0)) {
                        *draw_shadow = true;
                    }
                });

            canvas.want_capture_mouse()
        };

        if !capture {
            let input = ctx.shared::<InputSystem>();
            if let GesturePan::Move { movement, .. } = input.finger_pan() {
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
        }

        self.scene
            .update_material(self.material, "u_Ambient", *ambient)?;
        self.scene
            .update_material(self.material, "u_Diffuse", *diffuse)?;
        self.scene
            .update_material(self.material, "u_Specular", *specular)?;

        self.scene.advance(self.camera)?;

        if !self.draw_shadow {
            self.scene.draw(self.camera)?;
        } else {
            self.scene.draw_shadow(None)?;
        }
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.console.update(info);
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.width = 640;
    settings.window.height = 480;

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
