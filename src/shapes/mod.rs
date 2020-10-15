mod aabb;
mod box_shape;
mod geometry_wrapper;
mod hit_result;
mod invert_normal;
mod medium;
mod parabola;
mod rectangle;
mod shape;
mod shape_list;
mod sphere;
mod transform;
mod volume;

pub use crate::shapes;
pub use aabb::BoundingBox;
pub use box_shape::BoxShape;
pub use geometry_wrapper::{GeometryModifier, GeometryWrapper};
pub use hit_result::HitResult;
pub use medium::MediumDensity;
pub use shape::{CompoundShape, Shape, SimpleShape, UntransformedShape};
pub use shape_list::ShapeList;
pub use transform::Transformable;
pub use volume::Volume;

pub mod factories {
    use super::*;

    pub use box_shape::factories::*;
    pub use geometry_wrapper::factories::*;
    pub use invert_normal::factories::*;
    pub use medium::factories::*;
    pub use parabola::factories::*;
    pub use rectangle::factories::*;
    pub use sphere::factories::*;
    pub use volume::factories::*;
}
