use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::TracingStats;

use super::HitResult;

pub trait HittableClone {
    fn box_clone(&self) -> Box<dyn Hittable>;
}

pub trait Hittable: HittableClone + Sync + Send + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

impl<T: 'static + Hittable + Clone> HittableClone for T {
    fn box_clone(&self) -> Box<dyn Hittable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Hittable> {
    fn clone(&self) -> Box<dyn Hittable> {
        self.as_ref().box_clone()
    }
}
