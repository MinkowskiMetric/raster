use crate::math::*;
use crate::noise::Perlin;
use crate::{Color, Texture};
use std::{convert::TryInto, sync::Arc};

#[derive(Debug, Clone)]
pub struct NoiseTexture(Perlin, f64);

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self(Perlin::new(), scale)
    }

    pub fn perlin(&self) -> &Perlin {
        &self.0
    }

    pub fn scale(&self) -> f64 {
        self.1
    }
}

impl Texture for NoiseTexture {
    fn value(&self, p: Point3, _u: FloatType, _v: FloatType) -> Color {
        (Vector3::new(1.0, 1.0, 1.0)
            * (0.5
                * (1.0 + ((self.scale() * p.z) + (10.0 * self.perlin().turbulence(p, 7))).sin())))
        .try_into()
        .unwrap()
    }
}

pub mod factories {
    use super::*;

    pub fn noise_texture(scale: f64) -> Arc<NoiseTexture> {
        Arc::new(NoiseTexture::new(scale))
    }
}
