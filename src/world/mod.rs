use std::collections::HashMap;

use crate::{rendering::mesh::Mesh, world::chunk::Chunk};

/// World chunks, which contain block data
pub mod chunk;
/// Blocks
pub mod block;

pub type Coordinate = i32;

/// Holds state of the game world itself. Blocks, entities, whatever.
pub struct GameWorld {
    /// Currently loaded in chunks
    chunks: HashMap<(Coordinate, Coordinate), Chunk>
}

impl GameWorld {
    pub fn new() -> Self {
        let mut chunks = HashMap::new();
        for x in 0..20 {
            for y in 0..20 {
                chunks.insert((x,y), Chunk::new(x,y));
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
