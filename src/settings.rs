use cgmath::Point3;

use crate::physics::AABB;

/// The color of the skybox
pub const SKY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const SHADOW_RES: u32 = 8192;
pub const SHADOW_RENDER_SZ: f32 = 300.0;

pub const CAMERA_SPEED: f32 = 0.50;
pub const MOUSE_SENSITIVITY: f32 = 0.007;
pub const FOV: f32 = 70.0;

pub const CHUNK_SIZE: usize = 16;
/// The number of chunks to render away from the player
pub const RENDER_DIST: usize = 16;

pub const SEED: u32 = 613;

/// Ticks per second
pub const PHYSICS_TICK_RATE: f32 = 60.0;
pub const MOVE_SPEED: f32 = 4.0;
pub const PLAYER_AABB: AABB = AABB::new(
    0.6,
    1.8,
    0.6,
    Point3::new(0.3, 1.6, 0.3),
).unwrap();
