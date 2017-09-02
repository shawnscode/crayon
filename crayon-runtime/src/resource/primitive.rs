use graphics;

#[derive(Debug)]
pub struct Primitive {
    vertices: PrimitiveVertexData,
    indices: Option<PrimitiveIndexData>,
}

pub type PrimitiveVideoObject = (graphics::VertexBufferHandle, Option<graphics::IndexBufferHandle>);

impl Primitive {
    pub fn new(vertices: Vec<u8>, layout: graphics::VertexLayout, len: usize) -> Primitive {
        Primitive {
            vertices: PrimitiveVertexData::new(vertices, layout, len),
            indices: None,
        }
    }

    pub fn new_with_indices(vertices: Vec<u8>,
                            layout: graphics::VertexLayout,
                            len: usize,
                            indices: Vec<u8>,
                            format: graphics::IndexFormat,
                            ilen: usize)
                            -> Primitive {
        Primitive {
            vertices: PrimitiveVertexData::new(vertices, layout, len),
            indices: Some(PrimitiveIndexData::new(indices, format, ilen)),
        }
    }

    pub fn update_video_object(&mut self,
                               video: &mut graphics::Graphics)
                               -> graphics::errors::Result<()> {
        if self.vertices.vbo.is_none() {
            let vbo = video
                .create_vertex_buffer(&self.vertices.layout,
                                      graphics::ResourceHint::Static,
                                      self.vertices.len as u32,
                                      Some(&self.vertices.buf))?;
            self.vertices.vbo = Some(vbo);
        }

        if let Some(ref mut indices) = self.indices {
            if indices.ibo.is_none() {
                let ibo = video
                    .create_index_buffer(indices.format,
                                         graphics::ResourceHint::Static,
                                         indices.len as u32,
                                         Some(&indices.buf))?;
                indices.ibo = Some(ibo);
            }
        }

        Ok(())
    }

    pub fn video_object(&self) -> Option<PrimitiveVideoObject> {
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

impl super::Resource for Primitive {
    fn size(&self) -> usize {
        self.vertices.buf.len() +
        self.indices
            .as_ref()
            .and_then(|v| Some(v.buf.len()))
            .unwrap_or(0)
    }
}

#[derive(Debug)]
struct PrimitiveVertexData {
    pub buf: Vec<u8>,
    pub len: usize,
    pub layout: graphics::VertexLayout,
    pub vbo: Option<graphics::VertexBufferRef>,
}

impl PrimitiveVertexData {
    pub fn new(buf: Vec<u8>, layout: graphics::VertexLayout, len: usize) -> Self {
        PrimitiveVertexData {
            buf: buf,
            len: len,
            layout: layout,
            vbo: None,
        }
    }
}

#[derive(Debug)]
struct PrimitiveIndexData {
    pub buf: Vec<u8>,
    pub len: usize,
    pub format: graphics::IndexFormat,
    pub ibo: Option<graphics::IndexBufferRef>,
}

impl PrimitiveIndexData {
    pub fn new(buf: Vec<u8>, format: graphics::IndexFormat, len: usize) -> Self {
        PrimitiveIndexData {
            buf: buf,
            len: len,
            format: format,
            ibo: None,
        }
    }
}