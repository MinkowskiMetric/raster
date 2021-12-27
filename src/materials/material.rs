use crate::{constants, math::*, Color, GeometryHitResult, IntersectResult, Ray};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PartialScatterResult {
    pub attenuation: Vector3,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScatterResult {
    pub partial: PartialScatterResult,
    pub scattered: Ray,
}

#[derive(Clone, Debug)]
pub struct BaseMaterialScatterResult {
    pub emitted: Color,
    pub scatter: Option<ScatterResult>,
}

impl BaseMaterialScatterResult {
    pub fn split(self) -> (Color, Option<ScatterResult>) {
        (self.emitted, self.scatter)
    }
}

pub trait BaseMaterial: Sync + Send + std::fmt::Debug {
    fn base_scatter(
        &self,
        ray_in: &Ray,
        hit_record: GeometryHitResult,
    ) -> BaseMaterialScatterResult;
}

pub trait Material: Sync + Send + std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, hit_record: GeometryHitResult) -> Option<ScatterResult>;

    fn emitted(&self, _p: Point3, _uv: Point2) -> Color {
        constants::BLACK
    }
}

impl<T: Material> BaseMaterial for T {
    fn base_scatter(
        &self,
        ray_in: &Ray,
        hit_record: GeometryHitResult,
    ) -> BaseMaterialScatterResult {
        let emitted = self.emitted(hit_record.hit_point(), hit_record.uv());
        let scatter = self.scatter(ray_in, hit_record);

        BaseMaterialScatterResult { emitted, scatter }
    }
}
