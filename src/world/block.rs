use std::ops::{Deref, DerefMut};

use crate::world::chunk::Chunk;

/// The various block types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Water,
    Dirt,
    Grass,
    Sand,
    Stone,
    Log,
    Leaves,
}

/// One side of a block
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockSide {
    Front, Back,
    Left, Right,
    Top, Bottom,
}

impl BlockType {
    /// Returns the texture coords (x,y) of this block on the texture image
    ///
    /// Returns None if has no texture.
    pub fn texture(&self, side: BlockSide) -> Option<(u8, u8)> {
        match self {
            Self::Air => None,

            Self::Water => Some((3,0)),

            Self::Dirt => Some((0,0)),
            Self::Grass => match side {
                BlockSide::Top => Some((2,0)),
                BlockSide::Bottom => Some((0,0)),
                _ => Some((1,0)),
            },
            Self::Sand => Some((4,0)),
            Self::Stone => Some((5,0)),
            Self::Log => match side {
                BlockSide::Top | BlockSide::Bottom => Some((7,0)),
                _ => Some((6,0)),
            },
            Self::Leaves => Some((8,0)),
        }
    }

    /// Returns false for air and water, true otherwise.
    pub fn is_solid(&self) -> bool {
        match *self {
            Self::Air | Self::Water => false,
            _ => true,
        }
    }
}

/// A "smart pointer" to a block, such that when it goes out of scope, the 
/// chunk mesh is updated.
pub struct BlockRef<'a> {
    block: (usize, usize, usize),
    chunk: &'a mut Chunk,
    original_type: BlockType,
}

impl<'a> BlockRef<'a> {
    pub fn new(block: (usize, usize, usize), chunk: &'a mut Chunk) -> Self {
        let mut out = Self {
            block,
            chunk,
            original_type: BlockType::Air,
        };

        out.original_type = *out;
        out
    }
}

impl Drop for BlockRef<'_> {
    fn drop(&mut self) {
        if self.original_type != **self {
            self.chunk.update_mesh();
        }
    }
}

impl Deref for BlockRef<'_> {
    type Target = BlockType;

    fn deref(&self) -> &Self::Target {
        &self.chunk.blocks[self.block.0][self.block.2][self.block.1]
    }
}

impl DerefMut for BlockRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk.blocks[self.block.0][self.block.2][self.block.1]
    }
}
