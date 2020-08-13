mod aabb;
mod box_shape;
mod hit_result;
mod hittable;
mod invert_normal;
mod rectangle;
mod rotate;
mod scale;
mod shape_list;
mod sphere;
mod translate;
mod volume;

pub use crate::shapes;
pub use aabb::BoundingBox;
pub use hit_result::HitResult;
pub use hittable::Hittable;
pub use shape_list::ShapeList;
pub use volume::Volume;

pub mod factories {
    use super::*;

    pub use box_shape::factories::*;
    pub use invert_normal::factories::*;
    pub use rectangle::factories::*;
    pub use rotate::factories::*;
    pub use scale::factories::*;
    pub use shape_list::factories::*;
    pub use sphere::factories::*;
    pub use translate::factories::*;
    pub use volume::factories::*;
}

pub mod macro_pieces {
    use super::*;

    pub use shape_list::macro_pieces::*;
}
