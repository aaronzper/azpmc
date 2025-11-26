use cgmath::{Point3, Vector3};

#[derive(Debug)]
pub struct AABB {
    x_sz: f32,
    y_sz: f32,
    z_sz: f32,
    /// Where within the AABB the position of the parent entity is. (0,0,0) is
    /// the bottom corner of the box.
    origin: Point3<f32>,

    min_off: Vector3<f32>,
    max_off: Vector3<f32>,
}

impl AABB {
    /// Creates a new AABB with the given dimensions and centerpoint. Returns
    /// None if the centerpoint is outside of the dimensions.
    pub const fn new(x_sz: f32, y_sz: f32, z_sz: f32, origin: Point3<f32>) ->
        Option<Self> {

        if origin.x.is_sign_negative() || origin.y.is_sign_negative()
           || origin.z.is_sign_negative() || origin.x > x_sz || origin.y > y_sz
           || origin.z > z_sz {
            None
        } else {
            let min_off = Vector3 {
                x: -origin.x,
                y: -origin.y,
                z: -origin.z,
            };

            let max_off = Vector3 {
                x: x_sz - origin.x,
                y: y_sz - origin.y,
                z: z_sz - origin.z,
            };

            Some(Self {
                x_sz,
                y_sz,
                z_sz,
                origin,
                max_off,
                min_off,
            })
        }
    }

    /// Computes the bounds (opposite corners) of the bounding box at the given
    /// entity position
    pub fn get_bounds(&self, position: Point3<f32>) -> (Point3<f32>, Point3<f32>) {
        (position + self.min_off, position + self.max_off)
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, Point3::new(0.0, 0.0, 0.0)).unwrap()
    }
}
