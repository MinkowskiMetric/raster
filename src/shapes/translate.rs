use super::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::TracingStats;

#[derive(Debug, Clone)]
pub struct Translate<T: 'static + Hittable + Clone>(Vector3, T);

impl<T: 'static + Hittable + Clone> Translate<T> {
    pub fn new(offset: Vector3, child: T) -> Self {
        Self(offset, child)
    }

    fn offset(&self) -> Vector3 {
        self.0
    }

    fn child(&self) -> &T {
        &self.1
    }
}

impl<T: 'static + Hittable + Clone> Hittable for Translate<T> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let moved_ray = Ray::new(
            ray.origin.into_point() - self.offset(),
            ray.direction.into_vector(),
            ray.time,
        );
        if let Some(mut hit_result) = self.child().intersect(&moved_ray, t_min, t_max, stats) {
            hit_result.hit_point = hit_result.hit_point + self.offset();
            Some(hit_result)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let child_bounding_box = self.child().bounding_box(t0, t1);
        BoundingBox::new(
            (child_bounding_box.min_point().into_point() + self.offset()).into(),
            (child_bounding_box.max_point().into_point() + self.offset()).into(),
        )
    }
}

pub mod factories {
    use super::*;

    pub fn translate<T: 'static + Hittable + Clone>(offset: Vector3, child: T) -> Translate<T> {
        Translate::new(offset, child)
    }
}
