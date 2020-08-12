use super::{HitResult, Hittable, SharedHittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::TracingStats;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct InvertNormal(SharedHittable);

impl InvertNormal {
    pub fn new(child: SharedHittable) -> Self {
        Self(child)
    }

    fn child(&self) -> &dyn Hittable {
        self.0.as_ref()
    }
}

impl Hittable for InvertNormal {
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

    pub fn invert_normal(child: SharedHittable) -> Arc<InvertNormal> {
        Arc::new(InvertNormal::new(child))
    }
}
