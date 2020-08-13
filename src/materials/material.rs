use crate::color::Color;
use crate::constants;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::HitResult;

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
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult>;

    fn emitted(&self, _p: Point3, _u: FloatType, _v: FloatType) -> Color {
        constants::BLACK
    }
}
