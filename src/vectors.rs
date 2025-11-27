use cgmath::{Point3, Vector2, Vector3, num_traits::ToPrimitive};

use crate::world::ThreeDimPos;

#[derive(Debug)]
pub enum Dimension { X, Y, Z }

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

pub fn point_to_pos(p: Point3<f32>) -> ThreeDimPos {
    (
        p.x.floor().to_i32().unwrap(),
        p.y.floor().to_u8().expect("Y coordinate out of bounds"),
        p.z.floor().to_i32().unwrap(),
    )
}
