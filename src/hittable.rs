use crate::aabb::BoundingBox;
use crate::material::SharedMaterial;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::stats::TracingStats;
use std::sync::Arc;

pub struct HitResult<'a> {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub front_face: bool,
    pub material: &'a SharedMaterial,
    pub u: FloatType,
    pub v: FloatType,
}

pub trait Hittable: Sync + Send + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub type SharedHittable = Arc<dyn Hittable>;

pub mod shapes {
    use super::*;
    use crate::material::SharedMaterial;
    use crate::sphere::{MovingSphere, Sphere};

    pub fn sphere(center: Point3, radius: FloatType, material: SharedMaterial) -> Arc<Sphere> {
        Arc::new(Sphere::new(center, radius, material))
    }

    pub fn moving_sphere(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
        material: SharedMaterial,
    ) -> Arc<MovingSphere> {
        Arc::new(MovingSphere::new(center0, center1, radius, material))
    }
}
