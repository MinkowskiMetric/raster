use crate::color::Color;
use crate::math::*;
use crate::perlin::Perlin;
use std::convert::TryInto;
use std::sync::Arc;

pub trait Texture: Sync + Send + std::fmt::Debug {
    fn value(&self, p: Point3, u: FloatType, v: FloatType) -> Color;
}

pub type SharedTexture = Arc<dyn Texture>;

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

#[derive(Debug, Clone)]
pub struct CheckerTexture(SharedTexture, SharedTexture);

impl CheckerTexture {
    pub fn new(texture1: SharedTexture, texture2: SharedTexture) -> Self {
        Self(texture1, texture2)
    }

    pub fn texture1(&self) -> &dyn Texture {
        self.0.as_ref()
    }

    pub fn texture2(&self) -> &dyn Texture {
        self.1.as_ref()
    }
}

impl Texture for CheckerTexture {
    fn value(&self, p: Point3, u: FloatType, v: FloatType) -> Color {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();

        if sines < 0.0 {
            self.texture1().value(p, u, v)
        } else {
            self.texture2().value(p, u, v)
        }
    }
}

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

pub mod textures {
    use super::*;

    pub fn solid_texture(color: Color) -> Arc<SolidTexture> {
        Arc::new(SolidTexture::new(color))
    }

    pub fn checker_texture(
        texture1: SharedTexture,
        texture2: SharedTexture,
    ) -> Arc<CheckerTexture> {
        Arc::new(CheckerTexture::new(texture1, texture2))
    }

    pub fn noise_texture(scale: f64) -> Arc<NoiseTexture> {
        Arc::new(NoiseTexture::new(scale))
    }
}
