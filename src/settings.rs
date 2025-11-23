use crate::rendering::{textures::tex_cords_to_lin, vertex::Vertex};

/// The color of the skybox
pub const SKY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const CAMERA_SPEED: f32 = 0.05;

pub const TEST_MODEL_V: &[Vertex] = &[
    Vertex { position: [-0.25, -0.25, 0.0], texture_cords: tex_cords_to_lin(1, 1) }, // BL
    Vertex { position: [0.25, -0.25, 0.0],  texture_cords: tex_cords_to_lin(2, 1) }, // TL
    Vertex { position: [-0.25, 0.25, 0.0],  texture_cords: tex_cords_to_lin(1, 0) }, // BR
    Vertex { position: [0.25, 0.25, 0.0],   texture_cords: tex_cords_to_lin(2, 0) }, // TR
];

pub const TEST_MODEL_I: &[u16] = &[
    3, 2, 0,
    3, 0, 1
];
