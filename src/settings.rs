use crate::rendering::vertex::Vertex;

/// The color of the skybox
pub const SKY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

const fn tex_cords_to_lin(x: u8, y: u8) -> [f32; 2] {
    let multiplier = 1.0 / 256.0;
    [multiplier * x as f32, multiplier * -1.0 * y as f32]
}

pub const TEST_MODEL_V: &[Vertex] = &[
    Vertex { position: [-0.25, -0.25, 0.0], texture_cords: tex_cords_to_lin(0, 16) }, // BL
    Vertex { position: [0.25, -0.25, 0.0],  texture_cords: tex_cords_to_lin(16, 16) }, // TL
    Vertex { position: [-0.25, 0.25, 0.0],  texture_cords: tex_cords_to_lin(0, 0) }, // BR
    Vertex { position: [0.25, 0.25, 0.0],   texture_cords: tex_cords_to_lin(16, 0) }, // TR
];

pub const TEST_MODEL_I: &[u16] = &[
    3, 2, 0,
    3, 0, 1
];
