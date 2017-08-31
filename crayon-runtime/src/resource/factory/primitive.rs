use resource::errors::*;
use resource::{ResourceFrontend, Primitive, PrimitivePtr};

impl_vertex! {
    PrimitiveVertex {
        position => [Position; Float; 3; false],
        color => [Color0; UByte; 4; true],
        texcoord => [Texcoord0; Float; 2; false],
        normal => [Normal; Float; 3; false],
    }
}

const BUILTIN_CUBE_PATH: &'static str = "_CRAYON_/primitive/cube";

pub fn cube(frontend: &mut ResourceFrontend) -> Result<PrimitivePtr> {
    if let Some(rc) = frontend.get(BUILTIN_CUBE_PATH) {
        return Ok(rc);
    }

    use self::PrimitiveVertex as PV;

    let white = [255, 255, 255, 255];
    let corners = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let verts = vec![PV::new([-0.5, -0.5, -0.5], white, corners[0], [0.0, 0.0, -1.0]),
                     PV::new([0.5, -0.5, -0.5], white, corners[1], [0.0, 0.0, -1.0]),
                     PV::new([0.5, 0.5, -0.5], white, corners[2], [0.0, 0.0, -1.0]),
                     PV::new([0.5, 0.5, -0.5], white, corners[2], [0.0, 0.0, -1.0]),
                     PV::new([-0.5, 0.5, -0.5], white, corners[3], [0.0, 0.0, -1.0]),
                     PV::new([-0.5, -0.5, -0.5], white, corners[0], [0.0, 0.0, -1.0]),

                     PV::new([-0.5, -0.5, 0.5], white, corners[0], [0.0, 0.0, 1.0]),
                     PV::new([0.5, -0.5, 0.5], white, corners[1], [0.0, 0.0, 1.0]),
                     PV::new([0.5, 0.5, 0.5], white, corners[2], [0.0, 0.0, 1.0]),
                     PV::new([0.5, 0.5, 0.5], white, corners[2], [0.0, 0.0, 1.0]),
                     PV::new([-0.5, 0.5, 0.5], white, corners[3], [0.0, 0.0, 1.0]),
                     PV::new([-0.5, -0.5, 0.5], white, corners[0], [0.0, 0.0, 1.0]),

                     PV::new([-0.5, 0.5, 0.5], white, corners[0], [-1.0, 0.0, 0.0]),
                     PV::new([-0.5, 0.5, -0.5], white, corners[1], [-1.0, 0.0, 0.0]),
                     PV::new([-0.5, -0.5, -0.5], white, corners[2], [-1.0, 0.0, 0.0]),
                     PV::new([-0.5, -0.5, -0.5], white, corners[2], [-1.0, 0.0, 0.0]),
                     PV::new([-0.5, -0.5, 0.5], white, corners[3], [-1.0, 0.0, 0.0]),
                     PV::new([-0.5, 0.5, 0.5], white, corners[0], [-1.0, 0.0, 0.0]),

                     PV::new([0.5, 0.5, 0.5], white, corners[0], [1.0, 0.0, 0.0]),
                     PV::new([0.5, 0.5, -0.5], white, corners[1], [1.0, 0.0, 0.0]),
                     PV::new([0.5, -0.5, -0.5], white, corners[2], [1.0, 0.0, 0.0]),
                     PV::new([0.5, -0.5, -0.5], white, corners[2], [1.0, 0.0, 0.0]),
                     PV::new([0.5, -0.5, 0.5], white, corners[3], [1.0, 0.0, 0.0]),
                     PV::new([0.5, 0.5, 0.5], white, corners[0], [1.0, 0.0, 0.0]),

                     PV::new([-0.5, -0.5, -0.5], white, corners[0], [0.0, -1.0, 0.0]),
                     PV::new([0.5, -0.5, -0.5], white, corners[1], [0.0, -1.0, 0.0]),
                     PV::new([0.5, -0.5, 0.5], white, corners[2], [0.0, -1.0, 0.0]),
                     PV::new([0.5, -0.5, 0.5], white, corners[2], [0.0, -1.0, 0.0]),
                     PV::new([-0.5, -0.5, 0.5], white, corners[3], [0.0, -1.0, 0.0]),
                     PV::new([-0.5, -0.5, -0.5], white, corners[0], [0.0, -1.0, 0.0]),

                     PV::new([-0.5, 0.5, -0.5], white, corners[0], [0.0, 1.0, 0.0]),
                     PV::new([0.5, 0.5, -0.5], white, corners[1], [0.0, 1.0, 0.0]),
                     PV::new([0.5, 0.5, 0.5], white, corners[2], [0.0, 1.0, 0.0]),
                     PV::new([0.5, 0.5, 0.5], white, corners[2], [0.0, 1.0, 0.0]),
                     PV::new([-0.5, 0.5, 0.5], white, corners[3], [0.0, 1.0, 0.0]),
                     PV::new([-0.5, 0.5, -0.5], white, corners[0], [0.0, 1.0, 0.0])];

    let bytes = PrimitiveVertex::as_bytes(verts.as_slice()).to_vec();
    let cube = Primitive::new(bytes, PrimitiveVertex::layout(), verts.len());
    frontend.insert(BUILTIN_CUBE_PATH, cube)
}