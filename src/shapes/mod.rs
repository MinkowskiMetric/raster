mod box_shape;
mod medium;
mod mesh;
mod parabola;
mod rectangle;
mod sphere;

pub use medium::MediumDensity;
pub use mesh::{TriangleMesh, TriangleVertex};
pub use sphere::{MovingSphere, Sphere};

pub mod factories {
    use super::*;

    pub use box_shape::factories::*;
    pub use medium::factories::*;
    pub use mesh::factories::*;
    pub use parabola::factories::*;
    pub use rectangle::factories::*;
    pub use sphere::factories::*;
}
