use serde_json;
use uuid::Uuid;

use crate::application::ins::Inspectable;
use crate::video::prelude::VideoSystemShared;

#[derive(Copy, Clone, Debug, Serialize)]
pub struct VideoInspectResourceField {
    pub size: u32,
    pub rc: u32,
    pub uuid: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct VideoInspectFields {
    pub meshes: Vec<VideoInspectResourceField>,
    pub textures: Vec<VideoInspectResourceField>,
    pub render_textures: Vec<VideoInspectResourceField>,
}

impl Inspectable for VideoSystemShared {
    fn inspect(&self, buf: &mut Vec<u8>) {
        let mut fields = VideoInspectFields {
            textures: Vec::new(),
            meshes: Vec::new(),
            render_textures: Vec::new(),
        };

        self.textures.iter(|_, value, rc, uuid| {
            fields.textures.push(VideoInspectResourceField {
                size: value.format.size(value.dimensions),
                rc: rc,
                uuid: uuid,
            });
        });

        self.meshes.iter(|_, value, rc, uuid| {
            fields.meshes.push(VideoInspectResourceField {
                size: (value.vertex_buffer_len() + value.index_buffer_len()) as u32,
                rc: rc,
                uuid: uuid,
            });
        });

        {
            let render_textures = self.render_textures.read().unwrap();
            for (_, v) in render_textures.iter() {
                fields.render_textures.push(VideoInspectResourceField {
                    size: v.format.size(v.dimensions),
                    rc: 1,
                    uuid: None,
                });
            }
        }

        let json = serde_json::to_string(&fields).unwrap();
        buf.extend_from_slice(json.as_bytes());
    }
}
