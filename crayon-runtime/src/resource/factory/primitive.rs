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

    let color = [155, 155, 155, 255];
    let texcoords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let points = [[-1.0, -1.0, 1.0],
                  [1.0, -1.0, 1.0],
                  [1.0, 1.0, 1.0],
                  [-1.0, 1.0, 1.0],
                  [-1.0, -1.0, -1.0],
                  [1.0, -1.0, -1.0],
                  [1.0, 1.0, -1.0],
                  [-1.0, 1.0, -1.0]];

    let normals = [[0.0, 0.0, 1.0],
                   [1.0, 0.0, 0.0],
                   [0.0, 0.0, -1.0],
                   [-1.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0],
                   [0.0, -1.0, 0.0]];

    let verts = vec![PV::new(points[0], color, texcoords[0], normals[0]),
                     PV::new(points[1], color, texcoords[1], normals[0]),
                     PV::new(points[2], color, texcoords[2], normals[0]),
                     PV::new(points[2], color, texcoords[2], normals[0]),
                     PV::new(points[3], color, texcoords[3], normals[0]),
                     PV::new(points[0], color, texcoords[0], normals[0]),

                     PV::new(points[1], color, texcoords[0], normals[1]),
                     PV::new(points[5], color, texcoords[1], normals[1]),
                     PV::new(points[6], color, texcoords[2], normals[1]),
                     PV::new(points[6], color, texcoords[2], normals[1]),
                     PV::new(points[2], color, texcoords[3], normals[1]),
                     PV::new(points[1], color, texcoords[0], normals[1]),

                     PV::new(points[5], color, texcoords[0], normals[2]),
                     PV::new(points[4], color, texcoords[1], normals[2]),
                     PV::new(points[7], color, texcoords[2], normals[2]),
                     PV::new(points[7], color, texcoords[2], normals[2]),
                     PV::new(points[6], color, texcoords[3], normals[2]),
                     PV::new(points[5], color, texcoords[0], normals[2]),

                     PV::new(points[4], color, texcoords[0], normals[3]),
                     PV::new(points[0], color, texcoords[1], normals[3]),
                     PV::new(points[3], color, texcoords[2], normals[3]),
                     PV::new(points[3], color, texcoords[2], normals[3]),
                     PV::new(points[7], color, texcoords[3], normals[3]),
                     PV::new(points[4], color, texcoords[0], normals[3]),

                     PV::new(points[3], color, texcoords[0], normals[4]),
                     PV::new(points[2], color, texcoords[1], normals[4]),
                     PV::new(points[6], color, texcoords[2], normals[4]),
                     PV::new(points[6], color, texcoords[2], normals[4]),
                     PV::new(points[7], color, texcoords[3], normals[4]),
                     PV::new(points[3], color, texcoords[0], normals[4]),

                     PV::new(points[4], color, texcoords[0], normals[5]),
                     PV::new(points[5], color, texcoords[1], normals[5]),
                     PV::new(points[1], color, texcoords[2], normals[5]),
                     PV::new(points[1], color, texcoords[2], normals[5]),
                     PV::new(points[0], color, texcoords[3], normals[5]),
                     PV::new(points[4], color, texcoords[0], normals[5])];

    let bytes = PrimitiveVertex::as_bytes(&verts).to_vec();
    let cube = Primitive::new(bytes, PrimitiveVertex::layout(), verts.len());
    frontend.insert(BUILTIN_CUBE_PATH, cube)
}