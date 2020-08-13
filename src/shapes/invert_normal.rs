use super::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::TracingStats;

#[derive(Debug, Clone)]
pub struct InvertNormal<T: Hittable + Clone>(T);

impl<T: 'static + Hittable + Clone> InvertNormal<T> {
    pub fn new(child: T) -> Self {
        Self(child)
    }

    fn child(&self) -> &T {
        &self.0
    }
}

impl<T: 'static + Hittable + Clone> Hittable for InvertNormal<T> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        if let Some(mut hit_result) = self.child().intersect(&ray, t_min, t_max, stats) {
            hit_result.front_face = !hit_result.front_face;

            Some(hit_result)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.child().bounding_box(t0, t1)
    }
}

pub mod factories {
    use super::*;

    pub fn invert_normal<T: 'static + Hittable + Clone>(child: T) -> InvertNormal<T> {
        InvertNormal::new(child)
    }
}
