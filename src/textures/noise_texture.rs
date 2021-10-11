use crate::math::*;
use crate::noise::Perlin;
use crate::{Color, Texture};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct NoiseTexture(Perlin, FloatType);

impl NoiseTexture {
    pub fn new(scale: FloatType) -> Self {
        Self(Perlin::new(), scale)
    }

    pub fn perlin(&self) -> &Perlin {
        &self.0
    }

    pub fn scale(&self) -> FloatType {
        self.1
    }
}

impl Texture for NoiseTexture {
    fn value(&self, p: Point3, _uv: (FloatType, FloatType)) -> Color {
        (Vector3::new(1.0, 1.0, 1.0)
            * (0.5
                * (1.0 + ((self.scale() * p.z) + (10.0 * self.perlin().turbulence(p, 7))).sin())))
        .try_into()
        .unwrap()
    }
}

pub mod factories {
    use super::*;

    pub fn noise_texture(scale: FloatType) -> NoiseTexture {
        NoiseTexture::new(scale)
    }
}
