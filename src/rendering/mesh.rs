use wgpu::{Buffer, Device, RenderPass, util::DeviceExt};

use crate::rendering::vertex::Vertex;

/// A 3D mesh that can be rendered.
pub struct Mesh {
    pub verticies: Vec<Vertex>,
    pub indicies: Vec<u32>,

    vtx_buf: Option<Buffer>,
    idx_buf: Option<Buffer>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            verticies: vec![],
            indicies: vec![],
            vtx_buf: None,
            idx_buf: None,
        }
    }

    pub fn set_buffers(&mut self, device: &Device) {
        self.vtx_buf = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.verticies[..]),
                usage: wgpu::BufferUsages::VERTEX,
            }
        ));

        self.idx_buf = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Index Buffer"),
                contents: bytemuck::cast_slice(&self.indicies[..]),
                usage: wgpu::BufferUsages::INDEX,
            }
        ));
    }

    pub fn are_buffers_set(&self) -> bool {
        self.vtx_buf.is_some() && self.idx_buf.is_some()
    }

    pub fn draw(&self, render_pass: &mut RenderPass) {
        let (v_buf, i_buf) = match (&self.vtx_buf, &self.idx_buf) {
            (Some(v), Some(i)) => (v, i),
            _ => return,
        };

        render_pass.set_vertex_buffer(0, v_buf.slice(..));
        render_pass.set_index_buffer(i_buf.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indicies.len() as u32, 0, 0..1);
    }
}
