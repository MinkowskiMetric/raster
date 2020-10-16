mod aabb;
mod box_shape;
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
pub use medium::MediumDensity;
pub use primitive::{
    CompoundPrimitive, GeometryHitResult, IntoAggregatePrimitive, IntoPrimitive, Primitive,
    PrimitiveHitResult, SkinnablePrimitive, SkinnedPrimitive, UntransformedPrimitive,
};
pub use rectangle::TransformedXyRectangle;
pub use shape::{CollectionShape, CompoundShape, HitResult, Shape, SimpleShape};
pub use shape_list::ShapeList;
pub use sphere::{MovingSphere, Sphere};
pub use transform::{TransformablePrimitive, TransformableShape};
pub use volume::Volume;

pub mod factories {
    use super::*;

    pub use box_shape::factories::*;
    pub use medium::factories::*;
    pub use parabola::factories::*;
    pub use rectangle::factories::*;
    pub use sphere::factories::*;
    pub use volume::factories::*;
}
