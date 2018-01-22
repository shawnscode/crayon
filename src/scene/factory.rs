pub mod shader {
    use graphics;

    pub fn pbr() -> graphics::ShaderSetup {
        let attributes = graphics::AttributeLayout::build()
            .with(graphics::Attribute::Position, 4)
            .with(graphics::Attribute::Normal, 4)
            .with(graphics::Attribute::Texcoord0, 2)
            .finish();

        let mut render_state = graphics::RenderState::default();
        render_state.color_blend = Some((
            graphics::Equation::Add,
            graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
            graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha),
        ));

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = attributes;
        setup.render_state = render_state;

        setup.vs = include_str!("assets/pbr.vs").to_owned();
        setup.fs = include_str!("assets/pbr.fs").to_owned();

        let tt = graphics::UniformVariableType::Matrix4f;
        setup.uniform_variables.insert("u_MVPMatrix".into(), tt);

        let mv = "u_ModelViewMatrix".into();
        setup.uniform_variables.insert(mv, tt);
        setup.uniform_variables.insert("u_NormalMatrix".into(), tt);

        setup
    }

    pub fn undefined() -> graphics::ShaderSetup {
        let attributes = graphics::AttributeLayout::build()
            .with(graphics::Attribute::Position, 4)
            .finish();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = attributes;

        setup.vs = include_str!("assets/undefined.vs").to_owned();
        setup.fs = include_str!("assets/undefined.fs").to_owned();

        setup
    }
}
