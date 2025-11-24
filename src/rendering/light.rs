#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// A light in the scene
pub struct Light {
    pub position: [f32; 3],
    /// Due to uniforms requiring 16 byte (4 float) spacing, we need to use a
    /// padding field here
    _padding0: u32,
    pub color: [f32; 3],
    /// Ditto
    _padding1: u32,
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding0: 0,
            color,
            _padding1: 0,
        }
    }

    pub fn sun() -> Self {
        Self::new([1.0 * 1000.0, 1.0 * 1000.0, 0.0], [1.0, 1.0, 1.0])
    }
}
