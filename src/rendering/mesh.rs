use crate::rendering::vertex::Vertex;

pub struct Mesh {
    pub verticies: Vec<Vertex>,
    pub indicies: Vec<u32>,
}
