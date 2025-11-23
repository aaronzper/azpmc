use crate::rendering::{textures::tex_cords_to_lin, vertex::Vertex};

/// The color of the skybox
pub const SKY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const CAMERA_SPEED: f32 = 0.05;
pub const MOUSE_SENSITIVITY: f32 = 0.007;

pub const TEST_MODEL_V: &[Vertex] = &[
    Vertex { position: [-0.25, -0.25, 0.0], texture_cords: tex_cords_to_lin(1, 1) }, // BL
    Vertex { position: [0.25, -0.25, 0.0],  texture_cords: tex_cords_to_lin(2, 1) }, // TL
    Vertex { position: [-0.25, 0.25, 0.0],  texture_cords: tex_cords_to_lin(1, 0) }, // BR
    Vertex { position: [0.25, 0.25, 0.0],   texture_cords: tex_cords_to_lin(2, 0) }, // TR

    Vertex { position: [0.25, -0.25, 0.0], texture_cords: tex_cords_to_lin(1, 1) }, // BL
    Vertex { position: [0.25, 0.25, 0.0],  texture_cords: tex_cords_to_lin(1, 0) }, // TL
    Vertex { position: [0.25, -0.25, -0.5],  texture_cords: tex_cords_to_lin(2, 1) }, // BR
    Vertex { position: [0.25, 0.25, -0.5],   texture_cords: tex_cords_to_lin(2, 0) }, // TR

    Vertex { position: [-0.25, -0.75, 0.0], texture_cords: tex_cords_to_lin(0, 1) }, // BL
    Vertex { position: [0.25, -0.75, 0.0],  texture_cords: tex_cords_to_lin(1, 1) }, // TL
    Vertex { position: [-0.25, -0.25, 0.0],  texture_cords: tex_cords_to_lin(0, 0) }, // BR
    Vertex { position: [0.25, -0.25, 0.0],   texture_cords: tex_cords_to_lin(1, 0) }, // TR

    Vertex { position: [0.25, -0.75, 0.0], texture_cords: tex_cords_to_lin(0, 1) }, // BL
    Vertex { position: [0.25, -0.25, 0.0],  texture_cords: tex_cords_to_lin(0, 0) }, // TL
    Vertex { position: [0.25, -0.75, -0.5],  texture_cords: tex_cords_to_lin(1, 1) }, // BR
    Vertex { position: [0.25, -0.25, -0.5],   texture_cords: tex_cords_to_lin(1, 0) }, // TR
];

pub const TEST_MODEL_I: &[u16] = &[
    3, 2, 0,
    3, 0, 1,

    4, 6, 7,
    5, 4, 7,

    11, 10, 8,
    11, 8, 9,

    12, 14, 15,
    13, 12, 15,
];
