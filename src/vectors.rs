use cgmath::{Vector2, Vector3};

pub const GRAVITY_A: Vector3<f32> = Vector3::new(0.0, -9.8, 0.0);

pub fn xyz_to_xz(v: Vector3<f32>) -> Vector2<f32> {
    Vector2::new(v.x, v.z)
}

pub fn replace_xz(v_xyz: Vector3<f32>, v_xz: Vector2<f32>) -> Vector3<f32> {
    Vector3 {
        x: v_xz.x,
        y: v_xyz.y,
        z: v_xz.y,
    }
}
