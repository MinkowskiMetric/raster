use super::{HitResult, Hittable, SharedHittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::TracingStats;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Scale(Vector3, SharedHittable);

impl Scale {
    pub fn new(scale: Vector3, child: SharedHittable) -> Self {
        Self(scale, child)
    }

    fn scale(&self) -> &Vector3 {
        &self.0
    }

    fn child(&self) -> &dyn Hittable {
        self.1.as_ref()
    }

    fn unscale_point(&self, p: Point3) -> Point3 {
        let scale = self.scale();
        Point3::new(p.x / scale.x, p.y / scale.y, p.z / scale.z)
    }

    fn unscale_vector(&self, p: Vector3) -> Vector3 {
        let scale = self.scale();
        Vector3::new(p.x / scale.x, p.y / scale.y, p.z / scale.z)
    }

    fn scale_point(&self, p: Point3) -> Point3 {
        let scale = self.scale();
        Point3::new(p.x * scale.x, p.y * scale.y, p.z * scale.z)
    }

    fn scale_vector(&self, p: Vector3) -> Vector3 {
        let scale = self.scale();
        Vector3::new(p.x * scale.x, p.y * scale.y, p.z * scale.z)
    }
}

impl Hittable for Scale {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let scaled_origin = self.unscale_point(ray.origin.into_point());
        let scaled_direction = self.unscale_vector(ray.direction.into_vector());
        let scaled_ray = Ray::new(scaled_origin, scaled_direction, ray.time);

        if let Some(mut hit_result) = self.child().intersect(&scaled_ray, t_min, t_max, stats) {
            hit_result.hit_point = self.scale_point(hit_result.hit_point);
            hit_result.surface_normal = self.scale_vector(hit_result.surface_normal).normalize();
            hit_result.distance = (hit_result.hit_point - ray.origin.into_point()).magnitude();

            Some(hit_result)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let child_box = self.child().bounding_box(t0, t1);
        BoundingBox::new(
            self.scale_point(child_box.min_point().into_point()),
            self.scale_point(child_box.max_point().into_point()),
        )
    }
}

pub mod factories {
    use super::*;

    pub fn scale(scale: Vector3, child: SharedHittable) -> Arc<Scale> {
        Arc::new(Scale::new(scale, child))
    }
}
