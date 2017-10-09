use graphics;

#[derive(Debug)]
pub struct Mesh {
    vertices: MeshVertexData,
    indices: Option<MeshIndexData>,
}

pub type MeshVideoObject = (graphics::VertexBufferHandle, Option<graphics::IndexBufferHandle>);

impl Mesh {
    pub fn new(vertices: Vec<u8>, layout: graphics::VertexLayout, len: usize) -> Self {
        Mesh {
            vertices: MeshVertexData::new(vertices, layout, len),
            indices: None,
        }
    }

    pub fn new_with_indices(vertices: Vec<u8>,
                            layout: graphics::VertexLayout,
                            len: usize,
                            indices: Vec<u8>,
                            format: graphics::IndexFormat,
                            ilen: usize)
                            -> Self {
        Self {
            vertices: MeshVertexData::new(vertices, layout, len),
            indices: Some(MeshIndexData::new(indices, format, ilen)),
        }
    }

    pub fn update_video_object(&mut self,
                               video: &mut graphics::GraphicsSystem)
                               -> graphics::errors::Result<()> {
        if self.vertices.vbo.is_none() {
            let vbo = video
                .create_vertex_buffer(&self.vertices.layout,
                                      graphics::BufferHint::Static,
                                      self.vertices.size() as u32,
                                      Some(&self.vertices.buf))?;
            self.vertices.vbo = Some(vbo);
        }

        if let Some(ref mut indices) = self.indices {
            if indices.ibo.is_none() {
                let ibo = video
                    .create_index_buffer(indices.format,
                                         graphics::BufferHint::Static,
                                         indices.size() as u32,
                                         Some(&indices.buf))?;
                indices.ibo = Some(ibo);
            }
        }

        Ok(())
    }

    pub fn video_object(&self) -> Option<MeshVideoObject> {
        if let Some(ref vbo) = self.vertices.vbo {
            if let Some(ref indices) = self.indices {
                if let Some(ref ibo) = indices.ibo {
                    Some((vbo.handle, Some(ibo.handle)))
                } else {
                    None
                }
            } else {
                Some((vbo.handle, None))
            }
        } else {
            None
        }
    }

    pub fn layout(&self) -> &graphics::VertexLayout {
        &self.vertices.layout
    }

    pub fn vlen(&self) -> usize {
        self.vertices.len
    }

    pub fn ilen(&self) -> Option<usize> {
        self.indices.as_ref().map(|v| v.len)
    }
}

impl super::super::Resource for Mesh {
    fn size(&self) -> usize {
        self.vertices.buf.len() +
        self.indices
            .as_ref()
            .and_then(|v| Some(v.buf.len()))
            .unwrap_or(0)
    }
}

#[derive(Debug)]
struct MeshVertexData {
    pub buf: Vec<u8>,
    pub len: usize,
    pub layout: graphics::VertexLayout,
    pub vbo: Option<graphics::VertexBufferRef>,
}

impl MeshVertexData {
    pub fn new(buf: Vec<u8>, layout: graphics::VertexLayout, len: usize) -> Self {
        assert!(layout.stride() as usize * len == buf.len());

        MeshVertexData {
            buf: buf,
            len: len,
            layout: layout,
            vbo: None,
        }
    }

    pub fn size(&self) -> usize {
        self.layout.stride() as usize * self.len
    }
}

#[derive(Debug)]
struct MeshIndexData {
    pub buf: Vec<u8>,
    pub len: usize,
    pub format: graphics::IndexFormat,
    pub ibo: Option<graphics::IndexBufferRef>,
}

impl MeshIndexData {
    pub fn new(buf: Vec<u8>, format: graphics::IndexFormat, len: usize) -> Self {
        assert!(buf.len() == (format.size() * len));

        MeshIndexData {
            buf: buf,
            len: len,
            format: format,
            ibo: None,
        }
    }

    pub fn size(&self) -> usize {
        self.format.size() * self.len
    }
}