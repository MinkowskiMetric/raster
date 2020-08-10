use crate::aabb::BoundingBox;
use crate::material::SharedMaterial;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::stats::TracingStats;
use std::sync::Arc;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct XyRect {
    x0: FloatType,
    x1: FloatType,
    y0: FloatType,
    y1: FloatType,
    k: FloatType,
    material: SharedMaterial,
}

impl XyRect {
    pub fn new(
        x0: FloatType,
        x1: FloatType,
        y0: FloatType,
        y1: FloatType,
        k: FloatType,
        material: SharedMaterial,
    ) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            material,
        }
    }
}

impl Hittable for XyRect {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let ray_origin = ray.origin.into_point();
        let ray_direction = ray.direction.into_vector();

        let t = (self.k - ray_origin.z) / ray_direction.z;
        if t < t_min || t > t_max {
            return None;
        }

        let x = ray_origin.x + (t * ray_direction.x);
        let y = ray_origin.y + (t * ray_direction.y);
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vector3::new(0.0, 0.0, 1.0);
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let hit_point = ray_origin + (t * ray_direction);

        Some(HitResult {
            distance: t,
            hit_point: hit_point.into(),
            surface_normal: surface_normal.into(),
            front_face,
            material: &self.material,
            u,
            v,
        })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        BoundingBox::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        )
    }
}

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

    pub fn xy_rect(
        x0: FloatType,
        x1: FloatType,
        y0: FloatType,
        y1: FloatType,
        k: FloatType,
        material: SharedMaterial,
    ) -> Arc<XyRect> {
        Arc::new(XyRect::new(x0, x1, y0, y1, k, material))
    }
}
