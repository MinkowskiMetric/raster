use crate::math::*;
use crate::{Color, Texture};

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
    fn value(&self, _p: Point3, _uv: (FloatType, FloatType)) -> Color {
        *self.color()
    }
}

pub mod factories {
    use super::*;

    pub fn solid_texture(color: Color) -> SolidTexture {
        SolidTexture::new(color)
    }
}
