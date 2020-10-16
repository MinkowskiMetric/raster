use crate::color::Color;
use crate::math::*;

pub trait Texture: Sync + Send + std::fmt::Debug {
    fn value(&self, p: Point3, uv: (FloatType, FloatType)) -> Color;
}
