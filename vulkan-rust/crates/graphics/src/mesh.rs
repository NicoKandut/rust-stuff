use crate::Vertex;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
