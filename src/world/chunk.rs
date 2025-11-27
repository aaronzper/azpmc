use std::collections::HashMap;

use crate::{rendering::{mesh::Mesh, textures::tex_cords_to_lin, vertex::{NORMAL_BACK, NORMAL_DOWN, NORMAL_FRONT, NORMAL_LEFT, NORMAL_RIGHT, NORMAL_UP, Vertex}}, settings::CHUNK_SIZE, world::{Coordinate, ThreeDimPos, WorldPos, block::{BlockSide, BlockType}, generation::{sample_elevation, sample_tree}}};

const X: usize = CHUNK_SIZE;
const Y: usize = 256;
const Z: usize = CHUNK_SIZE;

/// Calculates the coordinats of the chunk that the given coordinates fall into
pub fn cords_to_chunk(position: WorldPos) -> WorldPos {
    fn cord_to_chunk(p: Coordinate) -> Coordinate {
        p - p.rem_euclid(CHUNK_SIZE as Coordinate)
    }

    (cord_to_chunk(position.0), cord_to_chunk(position.1))
}

/// Calculates the chunk-local coordinates of the given cord pair
pub fn cords_to_local(position: WorldPos) -> (usize, usize) {
    let (x, z) = position;

    let local_x = x.rem_euclid(CHUNK_SIZE as Coordinate) as usize;
    let local_z = z.rem_euclid(CHUNK_SIZE as Coordinate) as usize;

    (local_x, local_z)
}

/// An individual chunk containing block data and its own 3D mesh.
pub struct Chunk {
    pub(super) blocks: [[[BlockType; Y]; Z]; X],

    /// The world (block) position of the starting corner of the chunk
    pos: WorldPos,
    /// If a face of a block within this chunk should be highlighted, this
    /// contains the chunk-local block coordinate and the face of such block.
    highlighted: Option<(usize, usize, usize, BlockSide)>,

    pub(super) mesh: Mesh,
}

impl Chunk {
    /// Creates a new chunk, starting at world coordinates X and Z. Errors if
    /// X or Z are not divisible by `CHUNK_SIZE`.
    pub fn new(chunk_pos: WorldPos, scratch: &mut HashMap<ThreeDimPos, BlockType>) ->
        anyhow::Result<Self> {

        let (chunk_x, chunk_z) = chunk_pos;

        if chunk_x as usize % CHUNK_SIZE != 0 || chunk_z as usize % CHUNK_SIZE != 0 {
            anyhow::bail!("Provided chunk coordinates {} X and {} Z must be divisible by chunk size ({})",
                chunk_x, chunk_z, CHUNK_SIZE);
        }

        let mut blocks = [[[BlockType::Air; Y]; Z]; X];

        for x in 0..X {
            let w_x = (x as Coordinate) + chunk_x;
            for z in 0..Z {
                let w_z = (z as Coordinate) + chunk_z;

                let elevation = sample_elevation(w_x, w_z);
                let tree = elevation >= 64 && sample_tree(w_x, w_z);

                for y in 0..Y {
                    let pos_3d = (w_x, y as u8, w_z);
                    let scratch_block = scratch.remove(&pos_3d);

                    if scratch_block.is_some() {
                        blocks[x][z][y] = scratch_block.unwrap();
                        continue;
                    }

                    if y < elevation - 3 {
                        blocks[x][z][y] = BlockType::Stone
                    } else if y < elevation {
                        blocks[x][z][y] = BlockType::Dirt
                    } else if y == elevation && y <= 64 {
                        blocks[x][z][y] = BlockType::Sand;
                    } else if y == elevation {
                        blocks[x][z][y] = BlockType::Grass
                    } else if y > elevation && y <= 64 {
                        blocks[x][z][y] = BlockType::Water;
                    } else if y < elevation + 5 && tree {
                        blocks[x][z][y] = BlockType::Log;
                    } else if y == elevation + 5 && tree {
                        const LEAVES_DIM: isize = 3;
                        let start_x = x as isize - LEAVES_DIM;
                        let start_z = z as isize - LEAVES_DIM;
                        let end_x = x as isize + LEAVES_DIM + 1;
                        let end_z = z as isize + LEAVES_DIM + 1;

                        for leaf_x in start_x..end_x {
                            for leaf_z in start_z..end_z {
                                for leaf_y in y..y+3 {
                                    if leaf_x >= 0 && leaf_x < X as isize &&
                                       leaf_y < Y &&
                                       leaf_z >= 0 && leaf_z < Z as isize {
                                        blocks[leaf_x as usize][leaf_z as usize][leaf_y] =
                                            BlockType::Leaves;
                                    } else {
                                        let pos = (
                                            chunk_x + leaf_x as Coordinate,
                                            leaf_y as u8,
                                            chunk_z + leaf_z as Coordinate,
                                        );
                                        scratch.insert(pos, BlockType::Leaves);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut out = Self {
            blocks,
            pos: (chunk_x, chunk_z),
            highlighted:
                Some((0, sample_elevation(chunk_x, chunk_z), 0, BlockSide::Top)),
            mesh: Mesh::new(),
        };
        out.generate_mesh();

        Ok(out)
    }

    fn add_side(&mut self, x: usize, y: usize, z: usize, side: BlockSide) {
        // Cull sides that face other blocks
        if let Some((facing_x, facing_y, facing_z)) = match side {
            // TODO: Check blocks in adjacent chunks
            BlockSide::Bottom  if y > 0      => Some((x, y - 1, z)),
            BlockSide::Top   if y < Y - 1  => Some((x, y + 1, z)),
            BlockSide::Left   if x > 0      => Some((x - 1, y, z)),
            BlockSide::Right  if x < X - 1  => Some((x + 1, y, z)),
            BlockSide::Front if z > 0      => Some((x, y, z - 1)),
            BlockSide::Back    if z < Z - 1  => Some((x, y, z + 1)),
            _ => None,
        } {
            let facing = self.blocks[facing_x][facing_z][facing_y];
            if !facing.is_renderable_adjacent() || self.blocks[x][z][y] == facing {
                return;
            }
        } else if let BlockSide::Bottom = side {
            return;
        }

        let x_f = (x as Coordinate + self.pos.0) as f32;
        let z_f = (z as Coordinate + self.pos.1) as f32;
        let y_f = y as f32;
        let t_opt = self.blocks[x][z][y].texture(side);

        if let None = t_opt {
            return;
        }
        let (t_x, t_y) = t_opt.unwrap();

        let is_highlighted = matches!(
            self.highlighted,
            Some((h_x, h_y, h_z, h_s)) if
                h_x == x && h_y == y && h_z == z && h_s == side
        ) as u32;

        let verticies = match side {
            BlockSide::Front => [
                Vertex { // BL
                    position: [x_f, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_FRONT,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_FRONT,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f + 1.0, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_FRONT,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f + 1.0, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_FRONT,
                    is_highlighted,
                },
            ],

            BlockSide::Back => [
                Vertex { // BL
                    position: [x_f + 1.0, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_BACK,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f + 1.0, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_BACK,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_BACK,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_BACK,
                    is_highlighted,
                },
            ],


            BlockSide::Top => [
                Vertex { // BL
                    position: [x_f, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_UP,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_UP,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f + 1.0, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_UP,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f + 1.0, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_UP,
                    is_highlighted,
                },
            ],

            BlockSide::Bottom => [
                Vertex { // BL
                    position: [x_f, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_DOWN,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_DOWN,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f + 1.0, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_DOWN,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f + 1.0, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_DOWN,
                    is_highlighted,
                },
            ],

            BlockSide::Left => [
                Vertex { // BL
                    position: [x_f, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_LEFT,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_LEFT,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_LEFT,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_LEFT,
                    is_highlighted,
                },
            ],

            BlockSide::Right => [
                Vertex { // BL
                    position: [x_f + 1.0, y_f, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1),
                    normal: NORMAL_RIGHT,
                    is_highlighted,
                },
                Vertex { // TL
                    position: [x_f + 1.0, y_f + 1.0, z_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y),
                    normal: NORMAL_RIGHT,
                    is_highlighted,
                },
                Vertex { // BR
                    position: [x_f + 1.0, y_f, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1),
                    normal: NORMAL_RIGHT,
                    is_highlighted,
                },
                Vertex { // TR
                    position: [x_f + 1.0, y_f + 1.0, z_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y),
                    normal: NORMAL_RIGHT,
                    is_highlighted,
                },
            ],
        };

        let start_index = self.mesh.verticies.len() as u32;
        let indicies = [
            start_index + 3, start_index + 2, start_index,
            start_index + 3, start_index, start_index + 1
        ];

        self.mesh.verticies.extend(verticies);
        self.mesh.indicies.extend(indicies);
    }

    fn generate_mesh(&mut self) {
        for x in 0..X {
            for y in 0..Y {
                for z in 0..Z {
                    self.add_side(x, y, z, BlockSide::Front);
                    self.add_side(x, y, z, BlockSide::Back);
                    self.add_side(x, y, z, BlockSide::Top);
                    self.add_side(x, y, z, BlockSide::Bottom);
                    self.add_side(x, y, z, BlockSide::Left);
                    self.add_side(x, y, z, BlockSide::Right);
                }
            }
        }
    }

    /// Throws out any existing mesh and regenerates it
    pub fn update_mesh(&mut self) {
        self.mesh = Mesh::new();
        self.generate_mesh();
    }
}
