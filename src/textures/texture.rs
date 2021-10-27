use crate::color::Color;
use crate::math::*;

pub trait Texture: Sync + Send + std::fmt::Debug {
    fn value(&self, p: Point3, uv: Point2) -> Color;
}
