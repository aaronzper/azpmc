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
