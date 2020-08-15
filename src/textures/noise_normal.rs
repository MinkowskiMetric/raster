use crate::math::*;
use crate::noise::Perlin;
use crate::{Color, Texture};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct NoiseNormal(Perlin, Perlin, f64, f64);

impl NoiseNormal {
    pub fn new(scale: f64, depth: f64) -> Self {
        Self(Perlin::new(), Perlin::new(), scale, depth)
    }

    pub fn perlin_x(&self) -> &Perlin {
        &self.0
    }

    pub fn perlin_y(&self) -> &Perlin {
        &self.1
    }

    pub fn scale(&self) -> f64 {
        self.2
    }

    pub fn depth(&self) -> f64 {
        self.3
    }

    fn perlin_random_unit_vector(&self, p: Point3) -> Vector3 {
        let ar = self.perlin_x().noise(self.scale() * p);
        let a = (ar + 1.0) * constants::PI;
        let z = self.perlin_y().noise(self.scale() * p);
        let r = (1.0 - z * z).sqrt();
        vec3(r * a.cos(), r * a.sin(), z)
    }
}

impl Texture for NoiseNormal {
    fn value(&self, p: Point3, _u: FloatType, _v: FloatType) -> Color {
        let perlin = self.perlin_random_unit_vector(p) * self.depth();
        let normal = (vec3(0.0, 0.0, 1.0) + perlin).normalize();
        ((normal / 2.0) + vec3(0.5, 0.5, 0.5)).try_into().unwrap()
    }
}

pub mod factories {
    use super::*;

    pub fn noise_normal(scale: f64, depth: f64) -> NoiseNormal {
        NoiseNormal::new(scale, depth)
    }
}
