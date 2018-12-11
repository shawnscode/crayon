use super::super::assets::prelude::*;
use super::{UniformVar, Visitor};

use crate::errors::*;
use crate::math::prelude::{Aabb2, Vector2};

pub struct HeadlessVisitor {}

impl HeadlessVisitor {
    pub fn new() -> Self {
        HeadlessVisitor {}
    }
}

impl Visitor for HeadlessVisitor {
    unsafe fn create_surface(&mut self, _: SurfaceHandle, _: SurfaceParams) -> Result<()> {
        Ok(())
    }

    unsafe fn delete_surface(&mut self, _: SurfaceHandle) -> Result<()> {
        Ok(())
    }

    unsafe fn create_shader(
        &mut self,
        _: ShaderHandle,
        _: ShaderParams,
        _: &str,
        _: &str,
    ) -> Result<()> {
        Ok(())
    }

    unsafe fn delete_shader(&mut self, _: ShaderHandle) -> Result<()> {
        Ok(())
    }

    unsafe fn create_texture(
        &mut self,
        _: TextureHandle,
        _: TextureParams,
        _: Option<TextureData>,
    ) -> Result<()> {
        Ok(())
    }

    unsafe fn update_texture(&mut self, _: TextureHandle, _: Aabb2<u32>, _: &[u8]) -> Result<()> {
        Ok(())
    }

    unsafe fn delete_texture(&mut self, _: TextureHandle) -> Result<()> {
        Ok(())
    }

    unsafe fn create_render_texture(
        &mut self,
        _: RenderTextureHandle,
        _: RenderTextureParams,
    ) -> Result<()> {
        Ok(())
    }

    unsafe fn delete_render_texture(&mut self, _: RenderTextureHandle) -> Result<()> {
        Ok(())
    }

    unsafe fn create_mesh(
        &mut self,
        _: MeshHandle,
        _: MeshParams,
        _: Option<MeshData>,
    ) -> Result<()> {
        Ok(())
    }

    unsafe fn update_vertex_buffer(&mut self, _: MeshHandle, _: usize, _: &[u8]) -> Result<()> {
        Ok(())
    }

    unsafe fn update_index_buffer(&mut self, _: MeshHandle, _: usize, _: &[u8]) -> Result<()> {
        Ok(())
    }

    unsafe fn delete_mesh(&mut self, _: MeshHandle) -> Result<()> {
        Ok(())
    }

    unsafe fn bind(&mut self, _: SurfaceHandle, _: Vector2<u32>) -> Result<()> {
        Ok(())
    }

    unsafe fn draw(
        &mut self,
        _: ShaderHandle,
        _: MeshHandle,
        _: MeshIndex,
        _: &[UniformVar],
    ) -> Result<u32> {
        Ok(0)
    }

    unsafe fn update_surface_scissor(&mut self, _: SurfaceScissor) -> Result<()> {
        Ok(())
    }

    unsafe fn update_surface_viewport(&mut self, _: SurfaceViewport) -> Result<()> {
        Ok(())
    }

    unsafe fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    unsafe fn advance(&mut self) -> Result<()> {
        Ok(())
    }
}
