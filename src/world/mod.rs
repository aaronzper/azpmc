use std::collections::HashMap;
use anyhow::Context;
use crate::{rendering::mesh::Mesh, settings::CHUNK_SIZE, world::chunk::Chunk};

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
        let mut chunks = HashMap::new();
        for x in 0..50 {
            let world_x = x * CHUNK_SIZE as Coordinate;
            for y in 0..50 {
                let world_y = y * CHUNK_SIZE as Coordinate;
                chunks.insert(
                    (world_x, world_y),
                    Chunk::new(world_x, world_y)
                        .context("Failed to create Chunk").unwrap()
                );
            }
        }

        Self {
            chunks,
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
}
