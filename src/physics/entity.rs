use cgmath::{Point3, Vector3, Zero, num_traits::{Signed, ToPrimitive}};
use wgpu::wgc::MAX_VERTEX_BUFFERS;
use crate::{physics::{AABB, entity}, settings::PHYSICS_TICK_RATE, vectors::Dimension, world::{Coordinate, GameWorld, ThreeDimPos, block::BlockType}};

#[derive(Debug)]
/// A raw `Entity` that only has a position, velocity, accel, and AABB
pub struct RawEntity {
    /// Measured in meters
    position: Point3<f32>,
    /// Measured in m/s
    velocity: Vector3<f32>,
    /// Measured in m/s^2
    acceleration: Vector3<f32>,
    /// Bounding box, for collision detection
    bounding_box: AABB,
}   

/// A dynamic, physics-affected thing in-game (players, mobs, whatever)
pub trait Entity {
    /// Computes one physics tick of the entity
    fn tick(&mut self, world: &GameWorld);
    /// Gets the precise, float value of the entity's position (as opposed to
    /// the integer coordinates).
    fn get_precise_pos(&self) -> Point3<f32>;
    fn set_pos(&mut self, p: Point3<f32>);
    fn set_acceleration(&mut self, a: Vector3<f32>);
    fn get_velocity(&self) -> Vector3<f32>;
    fn set_velocity(&mut self, v: Vector3<f32>);

    /// Gets the integer world position of the entity
    fn get_world_pos(&self) -> ThreeDimPos {
        (
            self.get_precise_pos().x.floor() as Coordinate,
            self.get_precise_pos().y.floor() as u8,
            self.get_precise_pos().z.floor() as Coordinate,
        )
    }
}

impl RawEntity {
    pub fn new(position: Point3<f32>, bounding_box: AABB) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            bounding_box,
        }
    }
}

impl Entity for RawEntity {
    fn tick(&mut self, world: &GameWorld) {
        self.velocity += self.acceleration / PHYSICS_TICK_RATE;

        let tick_v = self.velocity / PHYSICS_TICK_RATE;
        let dx = tick_v.x;
        let dy = tick_v.y;
        let dz = tick_v.z;

        let mut check_axis_collisions = |axis: Dimension| {
            match axis {
                Dimension::X => self.position.x += dx,
                Dimension::Y => self.position.y += dy,
                Dimension::Z => self.position.z += dz,
            };
            let bounds = self.bounding_box.get_bounds(self.position);

            let pos_movement = match axis {
                Dimension::X => dx > 0.0,
                Dimension::Y => dy > 0.0,
                Dimension::Z => dz > 0.0,
            };

            let (entity_axis_min, entity_axis_max) = match axis {
                Dimension::X => (bounds.0.x, bounds.1.x),
                Dimension::Y => (bounds.0.y, bounds.1.y),
                Dimension::Z => (bounds.0.z, bounds.1.z),
            };

            let start_x = bounds.0.x.floor().to_i32().unwrap();
            let end_x = bounds.1.x.ceil().to_i32().unwrap();
            let start_y = bounds.0.y.floor().to_i32().unwrap();
            let end_y = bounds.1.y.ceil().to_i32().unwrap();
            let start_z = bounds.0.z.floor().to_i32().unwrap();
            let end_z = bounds.1.z.ceil().to_i32().unwrap();
            let mut max_overlap = 0.0;
            for y in start_y..end_y {
                if y > u8::MAX as i32 { break; }
                for x in start_x..end_x {
                    for z in start_z..end_z {
                        if let Some(b) = world.get_block((x, y as u8, z)) &&
                            b.is_solid() {

                            let block_axis = match axis {
                                Dimension::X => x as f32,
                                Dimension::Y => y as f32,
                                Dimension::Z => z as f32,
                            };

                            let overlap = if pos_movement {
                                entity_axis_max - block_axis
                            } else {
                                (block_axis + 1.0) - entity_axis_min
                            };
                            
                            if overlap > max_overlap {
                                max_overlap = overlap;
                            }
                        }
                    }
                }
            }

            if pos_movement {
                max_overlap *= -1.0;
            };

            match axis {
                Dimension::X => self.position.x += max_overlap,
                Dimension::Y => self.position.y += max_overlap,
                Dimension::Z => self.position.z += max_overlap,
            };

            // Reset gravity velocity if we're on ground
            if let Dimension::Y = axis && max_overlap > 0.0 && dy.is_negative() {
                self.velocity.y = 0.0;
            }
        };

        check_axis_collisions(Dimension::X);
        check_axis_collisions(Dimension::Y);
        check_axis_collisions(Dimension::Z);
    }

    fn get_precise_pos(&self) -> Point3<f32> {
        self.position
    }

    fn set_pos(&mut self, p: Point3<f32>) {
        self.position = p;
    }

    fn set_acceleration(&mut self, a: Vector3<f32>) {
        self.acceleration = a;
    }

    fn get_velocity(&self) -> Vector3<f32> {
        self.velocity
    }

    fn set_velocity(&mut self, v: Vector3<f32>) {
        self.velocity = v;
    }
}
