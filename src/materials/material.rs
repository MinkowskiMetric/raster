use crate::color::Color;
use crate::constants;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::PrimitiveHitResult;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PartialScatterResult {
    pub attenuation: Vector3,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScatterResult {
    pub partial: PartialScatterResult,
    pub scattered: Ray,
}

pub trait Material: Sync + Send + std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, hit_record: PrimitiveHitResult) -> Option<ScatterResult>;

    fn emitted(&self, _p: Point3, _uv: (FloatType, FloatType)) -> Color {
        constants::BLACK
    }
}
