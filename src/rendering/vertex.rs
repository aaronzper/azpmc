/// An individual 3D vertex, as passed to the GPU
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_cords: [f32; 2],
}

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
                }
            ]
        }
    }
}
