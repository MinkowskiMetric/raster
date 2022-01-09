use crate::color::Color;
use crate::math::*;

pub trait Texture {
    fn value(&self, p: Point3, uv: Point2) -> Color;
}
