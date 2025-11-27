use std::{cmp::min, collections::{HashMap, HashSet}, mem::take, time::{Duration, Instant}};
use cgmath::{InnerSpace, MetricSpace, Point2, Point3, Vector3, Zero, num_traits::{Signed, abs, real::Real}};
use crate::{physics::Entity, rendering::mesh::Mesh, settings::{CHUNK_SIZE, MAX_HIGHLIGHT_DIST, PHYSICS_TICK_RATE, PLAYER_AABB, RENDER_DIST}, vectors::point_to_pos, world::{block::{BlockSide, BlockType}, chunk::{Chunk, cords_to_chunk, cords_to_local}, generation::sample_elevation, player::Player}};

/// World chunks, which contain block data
pub mod chunk;
/// Blocks
pub mod block;
/// World generation
mod generation;
/// The player
mod player;

/// A lateral coordinate (X or Z)
pub type Coordinate = i32;
/// A lateral (X, Z) position in the world space
pub type WorldPos = (Coordinate, Coordinate);
/// A 3D (X, Y, Z) position in the world space
pub type ThreeDimPos = (Coordinate, u8, Coordinate);

/// Holds state of the game world itself. Blocks, entities, whatever.
pub struct GameWorld {
    /// Currently loaded in chunks
    chunks: HashMap<WorldPos, Chunk>,
    /// Blocks generated into adjacent chunks that are yet to be put into a
    /// chunk themselves
    block_scratch: HashMap<ThreeDimPos, BlockType>,
    /// The player
    player: Player,
    /// The last time a physics tick was calculated. Used for enforcing the tick
    /// rate
    last_tick: Instant,
    /// The highlighted block, if there is one
    highlight: Option<ThreeDimPos>,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            block_scratch: HashMap::new(),
            player: Player::new(),
            last_tick: Instant::now(),
            highlight: None,
        }
    }

    /// Returns the various meshses to be rendered
    pub fn get_meshes_mut(&mut self) -> Box<[&mut Mesh]> {
        let mut meshes = vec![];
        for (_, chunk) in &mut self.chunks {
            meshes.push(&mut chunk.mesh);
        }

        meshes.into()
    }

    pub fn update_chunks_to_player(&mut self) {
        const RADIUS: isize = (RENDER_DIST * CHUNK_SIZE) as isize;
        const RADIUS_SQ: f32 = (RADIUS * RADIUS) as f32;

        let (player_x, _, player_z) = self.player.get_world_pos();
        let player_chunk = cords_to_chunk((player_x, player_z));
        let player_chunk_pt =
            Point2::new(player_chunk.0 as f32, player_chunk.1 as f32);

        // Remove chunks no longer in range
        let to_remove: Vec<(i32, i32)> = self.chunks.keys()
            .filter(|this_chunk| {
                let this_chunk_pt =
                    Point2::new(this_chunk.0 as f32, this_chunk.1 as f32);
                let dist_sq = player_chunk_pt.distance2(this_chunk_pt);

                dist_sq >= RADIUS_SQ
            })
            .cloned()
            .collect();
        for k in to_remove {
            self.chunks.remove(&k);
        }

        // Generate new chunks
        let x_start = player_chunk.0 as isize - RADIUS + CHUNK_SIZE as isize;
        let x_end = player_chunk.0 as isize + RADIUS;
        let z_start = player_chunk.1 as isize - RADIUS + CHUNK_SIZE as isize;
        let z_end = player_chunk.1 as isize + RADIUS;
        for x in (x_start..x_end).step_by(CHUNK_SIZE) {
            let f_x = x as f32;
            let c_x = x as Coordinate;
            for z in (z_start..z_end).step_by(CHUNK_SIZE) {
                let f_z = z as f32;
                let pt = Point2::new(f_x, f_z);
                let dist_sq = player_chunk_pt.distance2(pt);

                if dist_sq <= RADIUS_SQ {
                    let c_z = z as Coordinate;
                    let pos = (c_x, c_z);
                    if !self.chunks.contains_key(&pos) {
                        self.chunks.insert(
                            pos,
                            Chunk::new(pos, &mut self.block_scratch).unwrap()
                        );
                    }
                }
            }
        }

        // Update blocks written to the scratch
        let mut dirty_chunks = HashSet::new();
        let removed_keys: Vec<ThreeDimPos> = self.block_scratch.iter()
            .filter_map(|(pos_3d, block)| {
                let (x, y, z) = *pos_3d;
                let pos_2d = (x, z);
                let chunk_pos = cords_to_chunk(pos_2d);

                if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                    dirty_chunks.insert(chunk_pos);

                    let (local_x, local_z) = cords_to_local((x, z));
                    chunk.blocks[local_x][local_z][y as usize] = *block;
                    Some(pos_3d)
                } else {
                    None
                }
            })
            .copied()
            .collect();

        for key in removed_keys {
            self.block_scratch.remove(&key);
        }

        for chunk in dirty_chunks {
            self.chunks.get_mut(&chunk).unwrap().update_mesh();
        }
    }

    pub fn get_block(&self, pos: ThreeDimPos) -> Option<BlockType> {
        let x = pos.0 as Coordinate;
        let y = pos.1 as usize;
        let z = pos.2 as Coordinate;

        let chunk_pos = cords_to_chunk((x, z));
        let (local_x, local_z) = cords_to_local((x, z));
        match self.chunks.get(&chunk_pos) {
            Some(chunk) => Some(chunk.blocks[local_x][local_z][y]),
            None => None,
        }
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Executes a physics tick for all entities
    pub fn do_tick(&mut self) {
        const TICK_DURATION: Duration =
            Duration::new(0, ((1.0 / PHYSICS_TICK_RATE) * 1.0e9) as u32);

        if self.last_tick.elapsed() < TICK_DURATION {
            return;
        }
        self.last_tick = Instant::now();

        self.cast_highlight();

        let mut player = take(&mut self.player);
        player.tick(self);
        self.player = player;
    }

    /// Updates the currently highlighted block face per the provided ray,
    /// starting from the player position
    fn cast_highlight(&mut self) {
        let ray = self.player.facing.normalize();

        if ray.magnitude().is_zero() {
            return;
        }

        let step_x = if ray.x.is_positive() { 1. } else { -1. };
        let step_y = if ray.y.is_positive() { 1. } else { -1. };
        let step_z = if ray.z.is_positive() { 1. } else { -1. };

        let d_x = abs(1.0 / ray.x);
        let d_y = abs(1.0 / ray.y);
        let d_z = abs(1.0 / ray.z);

        let mut pos = self.player.get_precise_pos();

        let x_next = pos.x.floor() + if ray.x.is_positive() { 1. } else { 0. };
        let y_next = pos.y.floor() + if ray.y.is_positive() { 1. } else { 0. };
        let z_next = pos.z.floor() + if ray.z.is_positive() { 1. } else { 0. };

        let mut t_max_x = (x_next - pos.x) / ray.x;
        let mut t_max_y = (y_next - pos.y) / ray.y;
        let mut t_max_z = (z_next - pos.z) / ray.z;

        let mut dist = 0;
        loop {
            let block = point_to_pos(pos);
            if let Some(b) = self.get_block(block) && b.is_solid() {
                self.highlight = Some(block);
                return;
            }

            if dist > MAX_HIGHLIGHT_DIST {
                self.highlight = None;
                return;
            }

            let min = t_max_x.min(t_max_y).min(t_max_z);
            if min == t_max_x {
                pos.x += step_x;
                t_max_x += d_x;
            } else if min == t_max_y {
                pos.y += step_y;
                t_max_y += d_y;
            } else { // t_max_z
                pos.z += step_z;
                t_max_z += d_z;
            }

            dist += 1;
        }
    }

    pub fn get_highlight(&self) -> Option<ThreeDimPos> {
        self.highlight
    }
}
