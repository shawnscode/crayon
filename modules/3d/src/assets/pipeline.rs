use std::collections::HashMap;

use crayon::graphics::assets::shader::{
    ShaderHandle, ShaderParams, ShaderSetup, UniformVariableType,
};
use crayon::resource::utils::location::Location;
use crayon::utils::HashValue;
use errors::{Error, Result};

impl_handle!(PipelineHandle);

pub type PipelineUniformLinks = HashMap<PipelineUniformVariable, HashValue<str>>;

#[derive(Debug, Clone, Default)]
pub struct PipelineSetup<'a> {
    shader: ShaderSetup<'a>,
    link_uniforms: PipelineUniformLinks,
}

impl<'a> PipelineSetup<'a> {
    pub fn new(shader: ShaderSetup<'a>) -> PipelineSetup<'a> {
        PipelineSetup {
            shader: shader,
            link_uniforms: HashMap::new(),
        }
    }

    pub fn link<T>(&mut self, k: PipelineUniformVariable, name: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref();

        if let Some(tt) = self.shader.params.uniforms.variable_type(name) {
            if tt != k.into() {
                return Err(Error::UniformMismatch);
            }
        } else {
            return Err(Error::UniformMismatch);
        }

        self.link_uniforms.insert(k, name.into());
        Ok(())
    }

    pub fn location(&self) -> Location {
        self.shader.location
    }
}

impl<'a> Into<(Location<'a>, ShaderSetup<'a>, PipelineUniformLinks)> for PipelineSetup<'a> {
    fn into(self) -> (Location<'a>, ShaderSetup<'a>, PipelineUniformLinks) {
        (self.shader.location, self.shader, self.link_uniforms)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PipelineParams {
    pub shader: ShaderHandle,
    pub shader_params: ShaderParams,
    pub link_uniforms: HashMap<PipelineUniformVariable, HashValue<str>>,
}

impl PipelineParams {
    pub fn new(shader: ShaderHandle, params: ShaderParams, links: PipelineUniformLinks) -> Self {
        PipelineParams {
            shader: shader,
            shader_params: params,
            link_uniforms: links,
        }
    }

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

        DirLitViewDir0 => [Vector3f, "scn_DirLitViewDir[0]"],
        DirLitColor0 => [Vector3f, "scn_DirLitColor[0]"],
        DirLitShadowSpaceMatrix0 => [Matrix4f, "scn_DirLitShadowSpaceMatrix[0]"],
        DirLitShadowTexture0 => [RenderTexture, "scn_DirLitShadowTexture[0]"],

        PointLitViewPos0 => [Vector3f, "scn_PointLitViewPos[0]"],
        PointLitAttenuation0 => [Vector3f, "scn_PointLitAttenuation[0]"],
        PointLitColor0 => [Vector3f, "scn_PointLitColor[0]"],

        PointLitViewPos1 => [Vector3f, "scn_PointLitViewPos[1]"],
        PointLitAttenuation1 => [Vector3f, "scn_PointLitAttenuation[1]"],
        PointLitColor1 => [Vector3f, "scn_PointLitColor[1]"],

        PointLitViewPos2 => [Vector3f, "scn_PointLitViewPos[2]"],
        PointLitAttenuation2 => [Vector3f, "scn_PointLitAttenuation[2]"],
        PointLitColor2 => [Vector3f, "scn_PointLitColor[2]"],
        
        PointLitViewPos3 => [Vector3f, "scn_PointLitViewPos[3]"],
        PointLitAttenuation3 => [Vector3f, "scn_PointLitAttenuation[3]"],
        PointLitColor3 => [Vector3f, "scn_PointLitColor[3]"],
    }
);

impl PipelineUniformVariable {
    pub const POINT_LIT_UNIFORMS: [[PipelineUniformVariable; 3]; 4] = [
        [
            PipelineUniformVariable::PointLitViewPos0,
            PipelineUniformVariable::PointLitColor0,
            PipelineUniformVariable::PointLitAttenuation0,
        ],
        [
            PipelineUniformVariable::PointLitViewPos1,
            PipelineUniformVariable::PointLitColor1,
            PipelineUniformVariable::PointLitAttenuation1,
        ],
        [
            PipelineUniformVariable::PointLitViewPos2,
            PipelineUniformVariable::PointLitColor2,
            PipelineUniformVariable::PointLitAttenuation2,
        ],
        [
            PipelineUniformVariable::PointLitViewPos3,
            PipelineUniformVariable::PointLitColor3,
            PipelineUniformVariable::PointLitAttenuation3,
        ],
    ];

    pub const DIR_LIT_UNIFORMS: [[PipelineUniformVariable; 4]; 1] = [[
        PipelineUniformVariable::DirLitViewDir0,
        PipelineUniformVariable::DirLitColor0,
        PipelineUniformVariable::DirLitShadowTexture0,
        PipelineUniformVariable::DirLitShadowSpaceMatrix0,
    ]];
}
