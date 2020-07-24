use crate::aabb::BoundingBox;
use crate::hittable::{HitResult, Hittable};
use crate::material::Material;
use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone, Debug)]
pub struct Sphere {
    center: Point3,
    radius: FloatType,
    bounding_box: BoundingBox,
    material: Box<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: FloatType, material: Box<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
            bounding_box: BoundingBox::new(
                center - vec3(radius, radius, radius),
                center + vec3(radius, radius, radius),
            ),
        }
    }
}

impl Hittable for Sphere {
    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<HitResult> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                });
            }
        }

        return None;
    }

    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}
