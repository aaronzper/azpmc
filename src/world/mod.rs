use std::collections::HashMap;
use anyhow::Context;
use cgmath::{EuclideanSpace, MetricSpace, Point2};
use crate::{rendering::mesh::Mesh, settings::{CHUNK_SIZE, RENDER_DIST}, world::chunk::{Chunk, cords_to_chunk}};

/// World chunks, which contain block data
pub mod chunk;
/// Blocks
pub mod block;
/// World generation
mod generation;

pub type Coordinate = i32;

/// Holds state of the game world itself. Blocks, entities, whatever.
pub struct GameWorld {
    /// Currently loaded in chunks
    chunks: HashMap<(Coordinate, Coordinate), Chunk>
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    /// Returns the various meshses to be rendered
    pub fn get_meshes_mut(&mut self) -> Box<[&mut Mesh]> {
        let mut meshes = vec![];
        for chunk in &mut self.chunks {
            meshes.push(&mut chunk.1.mesh);
        }

        meshes.into()
    }

    pub fn update_chunks_to_player(&mut self, player: (Coordinate, Coordinate)) {
        const RADIUS: isize = (RENDER_DIST * CHUNK_SIZE) as isize;
        const RADIUS_SQ: f32 = (RADIUS * RADIUS) as f32;

        let player_chunk = cords_to_chunk(player);
        let player_chunk_pt =
            Point2::new(player_chunk.0 as f32, player_chunk.1 as f32);

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

        let x_start = player_chunk.0 as isize - RADIUS + CHUNK_SIZE as isize;
        let x_end = player_chunk.0 as isize + RADIUS;
        let y_start = player_chunk.1 as isize - RADIUS + CHUNK_SIZE as isize;
        let y_end = player_chunk.1 as isize + RADIUS;
        for x in (x_start..x_end).step_by(CHUNK_SIZE) {
            let f_x = x as f32;
            let c_x = x as Coordinate;
            for y in (y_start..y_end).step_by(CHUNK_SIZE) {
                let f_y = y as f32;
                let pt = Point2::new(f_x, f_y);
                let dist_sq = player_chunk_pt.distance2(pt);

                if dist_sq <= RADIUS_SQ {
                    let c_y = y as Coordinate;
                    let pos = (c_x, c_y);
                    if !self.chunks.contains_key(&pos) {
                        self.chunks.insert(pos, Chunk::new(c_x, c_y).unwrap());
                    }
                }
            }
        }
    }
}
