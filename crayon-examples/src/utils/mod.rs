use crayon::prelude::*;

use image;
use image::GenericImage;

use obj;

use std;
use std::io;
use std::time::Duration;

mod console;
pub use self::console::ConsoleCanvas;

pub struct TextureParser {}

impl graphics::TextureParser for TextureParser {
    type Error = image::ImageError;

    fn parse(bytes: &[u8]) -> image::ImageResult<graphics::TextureData> {
        let dynamic = image::load_from_memory(bytes)?.flipv();
        let dimensions = dynamic.dimensions();
        Ok(graphics::TextureData {
            format: graphics::TextureFormat::U8U8U8U8,
            dimensions: (dimensions.0 as u16, dimensions.1 as u16),
            data: dynamic.to_rgba().into_raw(),
        })
    }
}

impl_vertex!{
    OBJVertex {
        position => [Position; Float; 4; false],
        color => [Color0; UByte; 4; true],
        texcoord => [Texcoord0; Float; 2; false],
        normal => [Normal; Float; 3; false],
    }
}

pub struct OBJParser {}

impl OBJParser {
    fn add(
        mut a: math::Vector3<f32>,
        mut b: math::Vector3<f32>,
        mut c: math::Vector3<f32>,
        verts: &mut Vec<OBJVertex>,
        idxes: &mut Vec<u16>,
    ) {
        // Converts from right-handed into left-handed coordinate system.
        a.z *= -1.0;
        b.z *= -1.0;
        c.z *= -1.0;

        let color = [255, 255, 255, 255];
        let n = math::Vector3::cross(b - c, a - c).normalize().into();

        idxes.push(verts.len() as u16);
        verts.push(OBJVertex::new(a.extend(1.0).into(), color, [0.0, 0.0], n));

        idxes.push(verts.len() as u16);
        verts.push(OBJVertex::new(b.extend(1.0).into(), color, [0.0, 0.0], n));

        idxes.push(verts.len() as u16);
        verts.push(OBJVertex::new(c.extend(1.0).into(), color, [0.0, 0.0], n));
    }
}

impl graphics::MeshParser for OBJParser {
    type Error = io::Error;

    fn parse(bytes: &[u8]) -> io::Result<graphics::MeshData> {
        let data: obj::Obj<obj::SimplePolygon> =
            obj::Obj::load_buf(&mut std::io::BufReader::new(bytes))?;

        let mut verts = Vec::new();
        let mut idxes = Vec::new();
        let mut meshes = Vec::new();
        for o in data.objects {
            for mesh in o.groups {
                meshes.push(idxes.len());
                for poly in mesh.polys {
                    match poly.len() {
                        3 => {
                            let a = data.position[poly[0].0].into();
                            let b = data.position[poly[1].0].into();
                            let c = data.position[poly[2].0].into();
                            OBJParser::add(a, b, c, &mut verts, &mut idxes);
                        }
                        4 => {
                            let a = data.position[poly[0].0].into();
                            let b = data.position[poly[1].0].into();
                            let c = data.position[poly[2].0].into();
                            OBJParser::add(a, b, c, &mut verts, &mut idxes);

                            let a = data.position[poly[0].0].into();
                            let b = data.position[poly[2].0].into();
                            let c = data.position[poly[3].0].into();
                            OBJParser::add(a, b, c, &mut verts, &mut idxes);
                        }
                        _ => unreachable!(),
                    };
                }
            }
        }

        Ok(graphics::MeshData {
            layout: OBJVertex::layout(),
            index_format: graphics::IndexFormat::U16,
            primitive: graphics::Primitive::Triangles,
            num_verts: verts.len(),
            num_idxes: idxes.len(),
            sub_mesh_offsets: meshes,
            verts: Vec::from(OBJVertex::encode(&verts)),
            idxes: Vec::from(graphics::IndexFormat::encode(&idxes)),
        })
    }
}

pub fn to_ms(duration: Duration) -> f32 {
    duration.as_secs() as f32 * 1000.0 + duration.subsec_nanos() as f32 / 1_000_000.0
}
