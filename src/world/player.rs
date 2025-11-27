use cgmath::{InnerSpace, Point3, Vector2, Vector3, Zero};
use crate::{physics::{Entity, RawEntity}, settings::{GRAVITY_A, JUMP_SPEED, MOVE_SPEED, PLAYER_AABB, SPRINT_MULTIPLIER}, vectors::{replace_xz, xyz_to_xz}, world::{GameWorld, generation::sample_elevation}};

pub struct Player {
    /// The inner physics entity determining position
    pub entity: RawEntity,
    pub facing: Vector3<f32>,

    pub w_pressed: bool,   
    pub a_pressed: bool,   
    pub s_pressed: bool,   
    pub d_pressed: bool,   
    pub jump: bool, 
    pub sprint: bool,
}

impl Player {
    pub fn new() -> Self {
        let y = (sample_elevation(0, 0) + 2) as f32;
        let mut entity = RawEntity::new(Point3::new(0.0, y, 0.0), PLAYER_AABB);
        entity.set_acceleration(GRAVITY_A);

        Self {
            entity,
            facing: Vector3::zero(),
            w_pressed: false,
            a_pressed: false,
            s_pressed: false,
            d_pressed: false,
            jump: false,
            sprint: false,
        }
    }
}

impl Entity for Player {
    fn tick(&mut self, world: &GameWorld) { 
        let mut desired_x = 0.0;
        let mut desired_z = 0.0;

        if self.w_pressed {
            desired_z += 1.0;
        }
        if self.a_pressed {
            desired_x -= 1.0;
        }
        if self.s_pressed {
            desired_z -= 1.0;
        }
        if self.d_pressed {
            desired_x += 1.0;
        }

        let forward = xyz_to_xz(self.facing).normalize();
        let right = Vector2::new(-forward.y, forward.x).normalize();

        let speed = if self.sprint {
            MOVE_SPEED * SPRINT_MULTIPLIER
        } else { 
            MOVE_SPEED
        };

        let new_xz = ((forward * desired_z) + (right * desired_x)) * speed;
        let mut new_xyz = replace_xz(self.entity.get_velocity(), new_xz);
        
        if self.jump {
            self.jump = false;
            new_xyz.y = JUMP_SPEED;
        }

        self.entity.set_velocity(new_xyz);


        self.entity.tick(world);
    }

    fn get_precise_pos(&self) -> Point3<f32> {
        self.entity.get_precise_pos()
    }

    fn set_pos(&mut self, p: Point3<f32>) {
        self.entity.set_pos(p);
    }

    fn get_velocity(&self) -> cgmath::Vector3<f32> {
        self.entity.get_velocity()
    }

    fn set_velocity(&mut self, v: cgmath::Vector3<f32>) {
        self.entity.set_velocity(v);
    }

    fn set_acceleration(&mut self, a: cgmath::Vector3<f32>) {
        self.entity.set_acceleration(a);
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
