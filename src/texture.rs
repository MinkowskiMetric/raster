use crate::color::Color;
use crate::math::*;
use crate::perlin::Perlin;
use std::convert::TryInto;

pub trait Texture: Sync + Send + TextureClone + std::fmt::Debug {
    fn value(&self, p: Point3, u: FloatType, v: FloatType) -> Color;
}

pub trait TextureClone {
    fn box_clone(&self) -> Box<dyn Texture>;
}

impl<T: Texture + Clone + 'static> TextureClone for T {
    fn box_clone(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Texture> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

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
pub struct CheckerTexture(Box<dyn Texture>, Box<dyn Texture>);

impl CheckerTexture {
    pub fn new(texture1: Box<dyn Texture>, texture2: Box<dyn Texture>) -> Self {
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
pub struct NoiseTexture(Perlin);

impl NoiseTexture {
    pub fn new() -> Self {
        Self(Perlin::new())
    }

    pub fn perlin(&self) -> &Perlin {
        &self.0
    }
}

impl Texture for NoiseTexture {
    fn value(&self, p: Point3, _u: FloatType, _v: FloatType) -> Color {
        (Vector3::new(1.0, 1.0, 1.0) * self.perlin().noise(p))
            .try_into()
            .unwrap()
    }
}
