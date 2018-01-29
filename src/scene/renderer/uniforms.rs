use graphics::UniformVariableType;
use utils::HashValue;

macro_rules! impl_scene_uniforms {
    ($name: ident { $fkey: ident => [$fuvt: ident, $ffield: tt], $($key: ident => [$uvt: ident, $field: tt], )* }) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $fkey = 0,
            $(
                $key,
            ) *
        }

        impl $name {
            pub const FIELDS: &'static [&'static str] = &[
                $ffield,
                $( $field, )*
            ];

            pub const TYPES: &'static [UniformVariableType] = &[
                UniformVariableType::$fuvt,
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

impl_scene_uniforms!(
    SceneUniformVariables {
        ModelMatrix => [Matrix4f, "scn_ModelMatrix"],
        ModelViewMatrix => [Matrix4f, "scn_ModelViewMatrix"],
        ModelViewProjectionMatrix => [Matrix4f, "scn_MVPMatrix"],
        ViewNormalMatrix => [Matrix4f, "scn_ViewNormalMatrix"],
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

impl SceneUniformVariables {
    pub const POINT_LIT_FIELDS: [[SceneUniformVariables; 3]; 4] = [
        [
            SceneUniformVariables::PointLightViewPos0,
            SceneUniformVariables::PointLightColor0,
            SceneUniformVariables::PointLightAttenuation0,
        ],
        [
            SceneUniformVariables::PointLightViewPos1,
            SceneUniformVariables::PointLightColor1,
            SceneUniformVariables::PointLightAttenuation1,
        ],
        [
            SceneUniformVariables::PointLightViewPos2,
            SceneUniformVariables::PointLightColor2,
            SceneUniformVariables::PointLightAttenuation2,
        ],
        [
            SceneUniformVariables::PointLightViewPos3,
            SceneUniformVariables::PointLightColor3,
            SceneUniformVariables::PointLightAttenuation3,
        ],
    ];
}