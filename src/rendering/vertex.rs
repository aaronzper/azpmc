/// An individual 3D vertex, as passed to the GPU
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_cords: [f32; 2],
    pub normal: [f32; 3],
    pub is_highlighted: u32,
}

pub const NORMAL_UP: [f32; 3] = [0.0, 1.0, 0.0];
pub const NORMAL_DOWN: [f32; 3] = [0.0, -1.0, 0.0];
pub const NORMAL_LEFT: [f32; 3] = [-1.0, 0.0, 0.0];
pub const NORMAL_RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
pub const NORMAL_FRONT: [f32; 3] = [0.0, 0.0, -1.0];
pub const NORMAL_BACK: [f32; 3] = [0.0, 0.0, 1.0];

impl Vertex {
    /// Provides a buffer layout for a buffer of this type
    pub fn desc_layout() -> wgpu::VertexBufferLayout<'static> {
        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/#so-what-do-i-do-with-it
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                }
            ]
        }
    }
}
