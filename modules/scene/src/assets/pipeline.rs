use std::collections::HashMap;
use std::sync::Arc;

use crayon::graphics::assets::shader::*;
use crayon::utils::HashValue;

impl_handle!(PipelineHandle);

pub struct PipelineSetup {
    pub shader: ShaderHandle,
    pub link_uniforms: HashMap<PipelineUniformVariable, HashValue<str>>,
}

impl PipelineSetup {
    pub fn new(shader: ShaderHandle) -> PipelineSetup {
        PipelineSetup {
            shader: shader,
            link_uniforms: HashMap::new(),
        }
    }
}

pub(crate) struct PipelineObject {
    pub shader: ShaderHandle,
    pub link_uniforms: HashMap<PipelineUniformVariable, HashValue<str>>,
    pub sso: Arc<ShaderStateObject>,
}

impl PipelineObject {
    #[inline]
    pub fn uniform_field(&self, uv: PipelineUniformVariable) -> HashValue<str> {
        self.link_uniforms
            .get(&uv)
            .cloned()
            .unwrap_or_else(|| PipelineUniformVariable::FIELDS[uv as usize].into())
    }
}

macro_rules! impl_pipeline_uniforms {
    ($name: ident { $head: ident => [$tt_head: ident, $field_head: tt], $($tails: ident => [$uvt: ident, $field: tt], )* }) => {
        /// A list of supported build-in uniform variables that would be filled when
        /// drawing `Scene`.
        ///
        /// Space coordinate system related variables like `LitPosition`, `LitDir` are
        /// defined in _View_ space (or _Eye_ space) for conveninent.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $head = 0,
            $(
                $tails,
            ) *
        }

        impl $name {
            pub const UNIFORMS: &'static [$name] = &[
                $name::$head,
                $(
                    $name::$tails,
                ) *
            ];

            pub const FIELDS: &'static [&'static str] = &[
                $field_head,
                $( $field, )*
            ];

            pub const TYPES: &'static [UniformVariableType] = &[
                UniformVariableType::$tt_head,
                $( UniformVariableType::$uvt, ) *
            ];
        }

        impl Into<HashValue<str>> for $name {
            fn into(self) -> HashValue<str> {
                Self::FIELDS[self as usize].into()
            }
        }

        impl Into<UniformVariableType> for $name {
            fn into(self) -> UniformVariableType {
                Self::TYPES[self as usize]
            }
        }
    };
}

impl_pipeline_uniforms!(
    PipelineUniformVariable {
        ModelMatrix => [Matrix4f, "scn_ModelMatrix"],
        ModelViewMatrix => [Matrix4f, "scn_ModelViewMatrix"],
        ModelViewProjectionMatrix => [Matrix4f, "scn_MVPMatrix"],
        ViewNormalMatrix => [Matrix4f, "scn_ViewNormalMatrix"],
        ShadowCasterSpaceMatrix => [Matrix4f, "scn_ShadowCasterSpaceMatrix"],
        ShadowTexture => [RenderTexture, "scn_ShadowTexture"],
        DirLightViewDir => [Vector3f, "scn_DirLightViewDir"],
        DirLightColor => [Vector3f, "scn_DirLightColor"],
        PointLightViewPos0 => [Vector3f, "scn_PointLightViewPos[0]"],
        PointLightViewPos1 => [Vector3f, "scn_PointLightViewPos[1]"],
        PointLightViewPos2 => [Vector3f, "scn_PointLightViewPos[2]"],
        PointLightViewPos3 => [Vector3f, "scn_PointLightViewPos[3]"],
        PointLightAttenuation0 => [Vector3f, "scn_PointLightAttenuation[0]"],
        PointLightAttenuation1 => [Vector3f, "scn_PointLightAttenuation[1]"],
        PointLightAttenuation2 => [Vector3f, "scn_PointLightAttenuation[2]"],
        PointLightAttenuation3 => [Vector3f, "scn_PointLightAttenuation[3]"],
        PointLightColor0 => [Vector3f, "scn_PointLightColor[0]"],
        PointLightColor1 => [Vector3f, "scn_PointLightColor[1]"],
        PointLightColor2 => [Vector3f, "scn_PointLightColor[2]"],
        PointLightColor3 => [Vector3f, "scn_PointLightColor[3]"],
    }
);

impl PipelineUniformVariable {
    pub const POINT_LIT_UNIFORMS: [[PipelineUniformVariable; 3]; 4] = [
        [
            PipelineUniformVariable::PointLightViewPos0,
            PipelineUniformVariable::PointLightColor0,
            PipelineUniformVariable::PointLightAttenuation0,
        ],
        [
            PipelineUniformVariable::PointLightViewPos1,
            PipelineUniformVariable::PointLightColor1,
            PipelineUniformVariable::PointLightAttenuation1,
        ],
        [
            PipelineUniformVariable::PointLightViewPos2,
            PipelineUniformVariable::PointLightColor2,
            PipelineUniformVariable::PointLightAttenuation2,
        ],
        [
            PipelineUniformVariable::PointLightViewPos3,
            PipelineUniformVariable::PointLightColor3,
            PipelineUniformVariable::PointLightAttenuation3,
        ],
    ];
}
