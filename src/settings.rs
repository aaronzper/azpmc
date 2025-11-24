/// The color of the skybox
pub const SKY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const SHADOW_RES: u32 = 8192;

pub const CAMERA_SPEED: f32 = 0.50;
pub const MOUSE_SENSITIVITY: f32 = 0.007;
pub const FOV: f32 = 70.0;
pub const CHUNK_SIZE: usize = 16;

pub const SEED: u32 = 613;
