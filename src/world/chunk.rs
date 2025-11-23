use crate::{rendering::{mesh::Mesh, textures::tex_cords_to_lin, vertex::Vertex}, settings::CHUNK_SIZE, world::{Coordinate, block::{BlockSide, BlockType}}};

const X: usize = CHUNK_SIZE;
const Y: usize = CHUNK_SIZE;
const Z: usize = 256;

/// An individual chunk containing block data and its own 3D mesh.
pub struct Chunk {
    blocks: [[[BlockType; Z]; Y]; X],

    pos_x: Coordinate,
    pos_y: Coordinate,

    pub mesh: Mesh,
}

impl Chunk {
    pub fn new(x: Coordinate, y: Coordinate) -> Self {
        let mut blocks = [[[BlockType::Air; Z]; Y]; X];

        for (x, row) in blocks.iter_mut().enumerate() {
            for (_, col) in row.iter_mut().enumerate() {
                for z in 0..Z {
                    let elevation = x + 52;

                    if z < elevation {
                        col[z] = BlockType::Dirt
                    } else if z == elevation {
                        col[z] = BlockType::Grass
                    } else if z > elevation && z < 65 {
                        col[z] = BlockType::Water;
                    }
                }
            }
        }

        let mut out = Self {
            blocks,
            pos_x: x,
            pos_y: y,
            mesh: Mesh::new(),
        };
        out.generate_mesh();

        out
    }

    fn add_side(&mut self, x: usize, y: usize, z: usize, side: BlockSide) {
        // Cull sides that face other blocks
        if let Some((facing_x, facing_y, facing_z)) = match side {
            // TODO: Check blocks in adjacent chunks
            BlockSide::Front  if y > 0      => Some((x, y - 1, z)),
            BlockSide::Back   if y < Y - 1  => Some((x, y + 1, z)),
            BlockSide::Left   if x > 0      => Some((x - 1, y, z)),
            BlockSide::Right  if x < X - 1  => Some((x + 1, y, z)),
            BlockSide::Bottom if z > 0      => Some((x, y, z - 1)),
            BlockSide::Top    if z < Z - 1  => Some((x, y, z + 1)),
            _ => None,
        } {
            let facing = self.blocks[facing_x][facing_y][facing_z];
            if !facing.is_renderable_adjacent() || self.blocks[x][y][z] == facing {
                return;
            }
        } else if let BlockSide::Bottom = side {
            return;
        }

        let x_f = (x + (self.pos_x as usize * CHUNK_SIZE)) as f32;
        let y_f = (y + (self.pos_y as usize * CHUNK_SIZE)) as f32;
        let z_f = z as f32;
        let t_opt = self.blocks[x][y][z].texture(side);

        if let None = t_opt {
            return;
        }
        let (t_x, t_y) = t_opt.unwrap();

        let verticies = match side {
            BlockSide::Front => [
                Vertex { // BL
                    position: [x_f, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f + 1.0, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f + 1.0, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
                },
            ],

            BlockSide::Back => [
                Vertex { // BL
                    position: [x_f + 1.0, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f + 1.0, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
                },
            ],


            BlockSide::Top => [
                Vertex { // BL
                    position: [x_f, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f + 1.0, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f + 1.0, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
                },
            ],

            BlockSide::Bottom => [
                Vertex { // BL
                    position: [x_f, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f + 1.0, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f + 1.0, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
                },
            ],

            BlockSide::Left => [
                Vertex { // BL
                    position: [x_f, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
                },
            ],

            BlockSide::Right => [
                Vertex { // BL
                    position: [x_f + 1.0, z_f, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y+1)
                },
                Vertex { // TL
                    position: [x_f + 1.0, z_f + 1.0, y_f],
                    texture_cords: tex_cords_to_lin(t_x+1, t_y)
                },
                Vertex { // BR
                    position: [x_f + 1.0, z_f, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y+1)
                },
                Vertex { // TR
                    position: [x_f + 1.0, z_f + 1.0, y_f + 1.0],
                    texture_cords: tex_cords_to_lin(t_x, t_y)
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
}
