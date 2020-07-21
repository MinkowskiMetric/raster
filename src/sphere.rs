use crate::material::Material;
use crate::ray_scanner::Ray;
use crate::scene::{AlignedBoundingBox, HitResult, Shape};
use cgmath::prelude::*;

pub struct Sphere {
    center: cgmath::Vector3<f32>,
    radius: f32,
    material: Box<dyn Material>,
    bounding_box: AlignedBoundingBox,
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32, material: Box<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
            bounding_box: AlignedBoundingBox::from_center_and_size(
                center,
                cgmath::vec3(radius * 2.0, radius * 2.0, radius * 2.0),
            ),
        }
    }
}

impl Shape for Sphere {
    fn bounding_box(&self) -> &AlignedBoundingBox {
        &self.bounding_box
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
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
                let surface_normal = if front_face { outward_normal } else { -outward_normal };
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
                let surface_normal = if front_face { outward_normal } else { -outward_normal };
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
}
