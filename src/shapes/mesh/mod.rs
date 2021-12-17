mod arch;

mod obj_file;
mod tri_mesh;
mod vertex;

pub use tri_mesh::{TriangleMesh, VertexTuple};
pub use vertex::TriangleVertex;

pub mod factories {
    pub use super::obj_file::factories::*;
    pub use super::tri_mesh::factories::*;
}
