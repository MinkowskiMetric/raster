mod aabb;
mod box_shape;
mod geometry_wrapper;
mod invert_normal;
mod medium;
mod parabola;
mod primitive;
mod rectangle;
mod shape;
mod shape_list;
mod sphere;
mod transform;
mod volume;

pub use crate::shapes;
pub use aabb::BoundingBox;
pub use geometry_wrapper::{GeometryModifier, GeometryWrapper};
pub use medium::MediumDensity;
pub use primitive::{
    CompoundPrimitive, IntoPrimitive, Primitive, PrimitiveHitResult, SkinnablePrimitive,
    SkinnedPrimitive, UntransformedPrimitive,
};
pub use rectangle::TransformedXyRectangle;
pub use shape::{CompoundShape, HitResult, Shape, SimpleShape, UntransformedShape};
pub use shape_list::ShapeList;
pub use sphere::{MovingSphere, Sphere};
pub use transform::{TransformablePrimitive, TransformableShape};
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
