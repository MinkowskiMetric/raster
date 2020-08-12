use crate::math::*;
use crate::{Color, Texture};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SolidTexture(Color);

impl SolidTexture {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    pub fn color(&self) -> &Color {
        &self.0
    }
}

impl Texture for SolidTexture {
    fn value(&self, _p: Point3, _u: FloatType, _v: FloatType) -> Color {
        *self.color()
    }
}

pub mod factories {
    use super::*;

    pub fn solid_texture(color: Color) -> Arc<SolidTexture> {
        Arc::new(SolidTexture::new(color))
    }
}
