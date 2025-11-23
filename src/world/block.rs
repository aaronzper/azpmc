/// The various block types
#[derive(Copy, Clone, Debug)]
pub enum BlockType {
    Air,
    Dirt,
    Grass,
}

/// One side of a block
#[derive(Copy, Clone, Debug)]
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

            Self::Dirt => Some((0,0)),
            Self::Grass => match side {
                BlockSide::Top => Some((2,0)),
                BlockSide::Bottom => Some((0,0)),
                _ => Some((1,0)),
            }
        }
    }
}
