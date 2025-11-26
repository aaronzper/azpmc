use cgmath::{Point3, Vector3, Zero};
use crate::{settings::PHYSICS_TICK_RATE, world::{Coordinate, ThreeDimPos}};

/// A dynamic, physics-affected thing in-game (players, mobs, whatever)
pub struct Entity {
    /// Measured in meters
    position: Point3<f32>,
    /// Measured in m/s
    velocity: Vector3<f32>,
    /// Measured in m/s^2
    acceleration: Vector3<f32>,
}   

impl Entity {
    pub fn new(position: Point3<f32>) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
        }
    }

    pub fn tick(&mut self) {
        self.velocity += self.acceleration / PHYSICS_TICK_RATE;
        self.position += self.velocity / PHYSICS_TICK_RATE;

        // TODO: Remove when we actually have collisions. This just here to
        // prevent me from falling thru the world forever while I test.
        if self.position.y < 0.0 {
            self.position = Point3 { x: self.position.x, y: 300.0, z: self.position.z };
            self.velocity.y = 0.0;
        }
    }

    /// Gets the precise, float value of the entity's position (as opposed to
    /// the integer coordinates).
    pub fn get_precise_pos(&self) -> Point3<f32> {
        self.position
    }

    /// Gets the integer world position of the entity
    pub fn get_world_pos(&self) -> ThreeDimPos {
        (
            self.get_precise_pos().x.round() as Coordinate,
            self.get_precise_pos().y.round() as u8,
            self.get_precise_pos().z.round() as Coordinate,
        )
    }

    pub fn set_acceleration(&mut self, a: Vector3<f32>) {
        self.acceleration = a;
    }

    pub fn get_velocity(&self) -> Vector3<f32> {
        self.velocity
    }

    pub fn set_velocity(&mut self, v: Vector3<f32>) {
        self.velocity = v;
    }
}
