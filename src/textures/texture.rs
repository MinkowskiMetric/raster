use crate::color::Color;
use crate::math::*;
use std::sync::Arc;

pub trait Texture: Sync + Send + std::fmt::Debug {
    fn value(&self, p: Point3, u: FloatType, v: FloatType) -> Color;
}

pub type SharedTexture = Arc<dyn Texture>;
