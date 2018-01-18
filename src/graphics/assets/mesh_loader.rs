use std;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use resource;
use graphics::assets::mesh::*;
use graphics::backend::frame::{DoubleFrame, PreFrameTask};

/// Parsed mesh from `MeshParser`.
pub struct MeshData {
    pub layout: VertexLayout,
    pub index_format: IndexFormat,
    pub primitive: Primitive,
    pub num_verts: usize,
    pub num_idxes: usize,
    pub sub_mesh_offsets: Vec<usize>,
    pub verts: Vec<u8>,
    pub idxes: Vec<u8>,
}

/// Parse bytes into texture.
pub trait MeshParser {
    type Error: std::error::Error + std::fmt::Debug;

    fn parse(bytes: &[u8]) -> std::result::Result<MeshData, Self::Error>;
}

#[doc(hidden)]
#[derive(PartialEq, Eq)]
pub(crate) enum MeshState {
    NotReady,
    Ready,
    Err(String),
}

#[doc(hidden)]
pub(crate) struct MeshLoader<T>
    where T: MeshParser
{
    handle: MeshHandle,
    setup: MeshSetup,
    state: Arc<RwLock<MeshState>>,
    frames: Arc<DoubleFrame>,
    _phantom: PhantomData<T>,
}

impl<T> MeshLoader<T>
    where T: MeshParser
{
    pub fn new(handle: MeshHandle,
               state: Arc<RwLock<MeshState>>,
               setup: MeshSetup,
               frames: Arc<DoubleFrame>)
               -> Self {
        MeshLoader {
            handle: handle,
            setup: setup,
            state: state,
            frames: frames,
            _phantom: PhantomData,
        }
    }
}

impl<T> resource::ResourceAsyncLoader for MeshLoader<T>
    where T: MeshParser + Send + Sync + 'static
{
    fn on_finished(mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
            Ok(bytes) => {
                match T::parse(bytes) {
                    Ok(mesh) => {
                        self.setup.layout = mesh.layout;
                        self.setup.index_format = mesh.index_format;
                        self.setup.primitive = mesh.primitive;
                        self.setup.num_verts = mesh.num_verts;
                        self.setup.num_idxes = mesh.num_idxes;
                        self.setup.sub_mesh_offsets = mesh.sub_mesh_offsets;

                        let mut frame = self.frames.front();
                        let vptr = Some(frame.buf.extend_from_slice(&mesh.verts));
                        let iptr = Some(frame.buf.extend_from_slice(&mesh.idxes));
                        let task = PreFrameTask::CreateMesh(self.handle, self.setup, vptr, iptr);
                        frame.pre.push(task);

                        MeshState::Ready
                    }
                    Err(error) => {
                        let error = format!("Failed to load mesh at {:?}.\n{:?}", path, error);
                        MeshState::Err(error)
                    }
                }
            }
            Err(error) => {
                let error = format!("Failed to load mesh at {:?}.\n{:?}", path, error);
                MeshState::Err(error)
            }
        };

        *self.state.write().unwrap() = state;
    }
}