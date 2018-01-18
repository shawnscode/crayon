use crayon::prelude::*;

use image;
use image::GenericImage;

use obj;

use std;
use std::io;
use std::time::Duration;

pub struct TextureParser {}

impl graphics::TextureParser for TextureParser {
    type Error = image::ImageError;

    fn parse(bytes: &[u8]) -> image::ImageResult<graphics::TextureData> {
        let dynamic = image::load_from_memory(&bytes)?.flipv();
        Ok(graphics::TextureData {
            format: graphics::TextureFormat::U8U8U8U8,
            dimensions: dynamic.dimensions(),
            data: dynamic.to_rgba().into_raw(),
        })
    }
}

impl_vertex!{
    OBJVertex {
        position => [Position; Float; 3; false],
    }
}
// texcoord => [Texcoord0; Float; 3; false],
// normal => [Normal; Float; 3; false],

pub struct OBJParser {}

impl graphics::MeshParser for OBJParser {
    type Error = io::Error;

    fn parse(bytes: &[u8]) -> io::Result<graphics::MeshData> {
        let data: obj::Obj<obj::SimplePolygon> =
            obj::Obj::load_buf(&mut std::io::BufReader::new(bytes))?;

        let mut verts = Vec::new();
        for v in data.position {
            verts.push(OBJVertex::new(v));
        }

        let mut idxes = Vec::new();
        let mut meshes = Vec::new();
        for o in data.objects {
            for mesh in o.groups {
                meshes.push(idxes.len());
                for poly in mesh.polys {
                    match poly.len() {
                        3 => {
                            idxes.push(poly[0].0 as u32);
                            idxes.push(poly[1].0 as u32);
                            idxes.push(poly[2].0 as u32);
                        }
                        4 => {
                            idxes.push(poly[0].0 as u32);
                            idxes.push(poly[1].0 as u32);
                            idxes.push(poly[2].0 as u32);

                            idxes.push(poly[0].0 as u32);
                            idxes.push(poly[2].0 as u32);
                            idxes.push(poly[3].0 as u32);
                        }
                        _ => unreachable!(),
                    };
                }
            }
        }

        Ok(graphics::MeshData {
            layout: OBJVertex::layout(),
            index_format: graphics::IndexFormat::U32,
            primitive: graphics::Primitive::Triangles,
            num_verts: verts.len(),
            num_idxes: idxes.len(),
            sub_mesh_offsets: meshes,
            verts: Vec::from(OBJVertex::as_bytes(&verts)),
            idxes: Vec::from(graphics::IndexFormat::as_bytes(&idxes)),
        })
    }
}

pub fn to_ms(duration: Duration) -> f32 {
    duration.as_secs() as f32 * 1000.0 + duration.subsec_nanos() as f32 / 1_000_000.0
}
