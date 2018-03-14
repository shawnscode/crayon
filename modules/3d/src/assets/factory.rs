pub mod pipeline {
    use crayon::graphics::assets::prelude::*;
    use crayon::resource::prelude::*;
    use self::UniformVariableType as UVT;

    use scene::Scene;
    use assets::pipeline::{PipelineHandle, PipelineSetup};
    use errors::*;

    pub const PBR: &str = "__Core/Scene/Shader/PBR";
    pub const PHONG: &str = "__Core/Scene/Shader/PHONG";
    pub const UNDEFINED: &str = "__Core/Scene/Shader/UNDEFINED";
    pub const COLOR: &str = "__Core/Scene/Shader/COLOR";

    pub fn pbr(scene: &mut Scene) -> Result<PipelineHandle> {
        let location = Location::shared(PBR);
        if let Some(pipeline) = scene.lookup_pipeline(location) {
            return Ok(pipeline);
        }

        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 4)
            .with(Attribute::Normal, 4)
            .with(Attribute::Texcoord0, 2)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("scn_MVPMatrix", UVT::Matrix4f)
            .with("scn_ModelViewMatrix", UVT::Matrix4f)
            .with("scn_ViewNormalMatrix", UVT::Matrix4f)
            .finish();

        let mut render_state = RenderState::default();
        render_state.depth_write = true;
        render_state.depth_test = Comparison::LessOrEqual;

        let mut setup = ShaderSetup::default();
        setup.location = location;
        setup.vs = include_str!("../../assets/pbr.vs").to_owned();
        setup.fs = include_str!("../../assets/pbr.fs").to_owned();

        setup.params.render_state = render_state;
        setup.params.attributes = attributes;
        setup.params.uniforms = uniforms;

        let pipeline_setup = PipelineSetup::new(setup);
        scene.create_pipeline(pipeline_setup)
    }

    pub fn phong(scene: &mut Scene) -> Result<PipelineHandle> {
        let location = Location::shared(PHONG);
        if let Some(pipeline) = scene.lookup_pipeline(location) {
            return Ok(pipeline);
        }

        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 3)
            .with(Attribute::Normal, 3)
            .with(Attribute::Color0, 4)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("scn_MVPMatrix", UVT::Matrix4f)
            .with("scn_ModelViewMatrix", UVT::Matrix4f)
            .with("scn_ViewNormalMatrix", UVT::Matrix4f)
            .with("scn_DirLitShadowSpaceMatrix[0]", UVT::Matrix4f)
            .with("scn_DirLitShadowTexture[0]", UVT::RenderTexture)
            .with("scn_DirLitViewDir[0]", UVT::Vector3f)
            .with("scn_DirLitColor[0]", UVT::Vector3f)
            .with("scn_PointLitViewPos[0]", UVT::Vector3f)
            .with("scn_PointLitColor[0]", UVT::Vector3f)
            .with("scn_PointLitAttenuation[0]", UVT::Vector3f)
            .with("scn_PointLitViewPos[1]", UVT::Vector3f)
            .with("scn_PointLitColor[1]", UVT::Vector3f)
            .with("scn_PointLitAttenuation[1]", UVT::Vector3f)
            .with("scn_PointLitViewPos[2]", UVT::Vector3f)
            .with("scn_PointLitColor[2]", UVT::Vector3f)
            .with("scn_PointLitAttenuation[2]", UVT::Vector3f)
            .with("scn_PointLitViewPos[3]", UVT::Vector3f)
            .with("scn_PointLitColor[3]", UVT::Vector3f)
            .with("scn_PointLitAttenuation[3]", UVT::Vector3f)
            .with("u_Ambient", UVT::Vector3f)
            .with("u_Diffuse", UVT::Vector3f)
            .with("u_Specular", UVT::Vector3f)
            .with("u_Shininess", UVT::F32)
            .finish();

        let mut render_state = RenderState::default();
        render_state.depth_write = true;
        render_state.depth_test = Comparison::LessOrEqual;
        render_state.cull_face = CullFace::Back;

        let mut setup = ShaderSetup::default();
        setup.location = location;
        setup.vs = include_str!("../../assets/phong.vs").to_owned();
        setup.fs = include_str!("../../assets/phong.fs").to_owned();

        setup.params.render_state = render_state;
        setup.params.attributes = attributes;
        setup.params.uniforms = uniforms;

        let pipeline_setup = PipelineSetup::new(setup);
        scene.create_pipeline(pipeline_setup)
    }

    pub fn color(scene: &mut Scene) -> Result<PipelineHandle> {
        let location = Location::shared(COLOR);
        if let Some(pipeline) = scene.lookup_pipeline(location) {
            return Ok(pipeline);
        }

        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 3)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("scn_MVPMatrix", UVT::Matrix4f)
            .with("u_Color", UVT::Vector4f)
            .finish();

        let mut render_state = RenderState::default();
        render_state.depth_write = true;
        render_state.depth_test = Comparison::LessOrEqual;
        render_state.cull_face = CullFace::Back;

        let mut setup = ShaderSetup::default();
        setup.location = location;
        setup.vs = include_str!("../../assets/color.vs").to_owned();
        setup.fs = include_str!("../../assets/color.fs").to_owned();

        setup.params.render_state = render_state;
        setup.params.attributes = attributes;
        setup.params.uniforms = uniforms;

        let pipeline_setup = PipelineSetup::new(setup);
        scene.create_pipeline(pipeline_setup)
    }

    pub fn undefined(scene: &mut Scene) -> Result<PipelineHandle> {
        let location = Location::shared(UNDEFINED);
        if let Some(pipeline) = scene.lookup_pipeline(location) {
            return Ok(pipeline);
        }

        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 3)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("scn_MVPMatrix", UVT::Matrix4f)
            .finish();

        let mut render_state = RenderState::default();
        render_state.depth_write = true;
        render_state.depth_test = Comparison::LessOrEqual;
        render_state.cull_face = CullFace::Back;

        let mut setup = ShaderSetup::default();
        setup.location = location;
        setup.vs = include_str!("../../assets/undefined.vs").to_owned();
        setup.fs = include_str!("../../assets/undefined.fs").to_owned();

        setup.params.render_state = render_state;
        setup.params.attributes = attributes;
        setup.params.uniforms = uniforms;

        let pipeline_setup = PipelineSetup::new(setup);
        scene.create_pipeline(pipeline_setup)
    }
}

pub mod mesh {
    use crayon::graphics::errors::*;
    use crayon::graphics::prelude::*;
    use crayon::graphics::assets::prelude::*;
    use crayon::resource::prelude::*;
    use crayon::math;

    impl_vertex! {
        PrimitiveVertex {
            position => [Position; Float; 3; false],
            color => [Color0; UByte; 4; true],
            texcoord => [Texcoord0; Float; 2; false],
            normal => [Normal; Float; 3; false],
        }
    }

    pub const QUAD: &str = "__Core/Scene/Mesh/QUAD";
    pub const CUBE: &str = "__Core/Scene/Mesh/CUBE";

    pub fn quad(video: &GraphicsSystemShared) -> Result<MeshHandle> {
        let location = Location::shared(QUAD);
        if let Some(quad) = video.lookup_mesh(location) {
            return Ok(quad);
        }

        let color = [155, 155, 155, 255];
        let texcoords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let points = [
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];

        let normals = [[0.0, 0.0, -1.0]];

        let verts = [
            PrimitiveVertex::new(points[0], color, texcoords[0], normals[0]),
            PrimitiveVertex::new(points[1], color, texcoords[1], normals[0]),
            PrimitiveVertex::new(points[2], color, texcoords[2], normals[0]),
            PrimitiveVertex::new(points[3], color, texcoords[3], normals[0]),
        ];

        let idxes = [0, 1, 2, 2, 3, 0];

        let mut setup = MeshSetup::default();
        setup.location = location;
        setup.params.layout = PrimitiveVertex::layout();
        setup.params.num_verts = verts.len();
        setup.params.num_idxes = idxes.len();
        setup.params.sub_mesh_offsets.push(0);
        setup.verts = Some(PrimitiveVertex::encode(&verts));
        setup.idxes = Some(IndexFormat::encode::<u16>(&idxes));
        video.create_mesh(setup)
    }

    pub fn cube(video: &GraphicsSystemShared) -> Result<MeshHandle> {
        let location = Location::shared(CUBE);
        if let Some(cube) = video.lookup_mesh(location) {
            return Ok(cube);
        }

        let color = [155, 155, 155, 255];
        let texcoords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let points = [
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        ];

        let normals = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];

        let verts = [
            PrimitiveVertex::new(points[0], color, texcoords[0], normals[0]),
            PrimitiveVertex::new(points[1], color, texcoords[1], normals[0]),
            PrimitiveVertex::new(points[2], color, texcoords[2], normals[0]),
            PrimitiveVertex::new(points[3], color, texcoords[3], normals[0]),
            PrimitiveVertex::new(points[1], color, texcoords[0], normals[1]),
            PrimitiveVertex::new(points[5], color, texcoords[1], normals[1]),
            PrimitiveVertex::new(points[6], color, texcoords[2], normals[1]),
            PrimitiveVertex::new(points[2], color, texcoords[3], normals[1]),
            PrimitiveVertex::new(points[5], color, texcoords[0], normals[2]),
            PrimitiveVertex::new(points[4], color, texcoords[1], normals[2]),
            PrimitiveVertex::new(points[7], color, texcoords[2], normals[2]),
            PrimitiveVertex::new(points[6], color, texcoords[3], normals[2]),
            PrimitiveVertex::new(points[4], color, texcoords[0], normals[3]),
            PrimitiveVertex::new(points[0], color, texcoords[1], normals[3]),
            PrimitiveVertex::new(points[3], color, texcoords[2], normals[3]),
            PrimitiveVertex::new(points[7], color, texcoords[3], normals[3]),
            PrimitiveVertex::new(points[3], color, texcoords[0], normals[4]),
            PrimitiveVertex::new(points[2], color, texcoords[1], normals[4]),
            PrimitiveVertex::new(points[6], color, texcoords[2], normals[4]),
            PrimitiveVertex::new(points[7], color, texcoords[3], normals[4]),
            PrimitiveVertex::new(points[4], color, texcoords[0], normals[5]),
            PrimitiveVertex::new(points[5], color, texcoords[1], normals[5]),
            PrimitiveVertex::new(points[1], color, texcoords[2], normals[5]),
            PrimitiveVertex::new(points[0], color, texcoords[3], normals[5]),
        ];

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let idxes = [
            0, 1, 2, 2, 3, 0,
            4, 5, 6, 6, 7, 4,
            8, 9, 10, 10, 11, 8,
            12, 13, 14, 14, 15, 12,
            16, 17, 18, 18, 19, 16,
            20, 21, 22, 22, 23, 20,
        ];

        let mut setup = MeshSetup::default();
        setup.location = location;
        setup.params.layout = PrimitiveVertex::layout();
        setup.params.num_verts = verts.len();
        setup.params.num_idxes = idxes.len();
        setup.params.sub_mesh_offsets.push(0);
        setup.params.aabb = math::Aabb3::new([-0.5, -0.5, -0.5].into(), [0.5, 0.5, 0.5].into());
        setup.verts = Some(PrimitiveVertex::encode(&verts));
        setup.idxes = Some(IndexFormat::encode::<u16>(&idxes));
        video.create_mesh(setup)
    }
}
