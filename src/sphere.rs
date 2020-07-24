use crate::aabb::BoundingBox;
use crate::hittable::{HitResult, Hittable};
use crate::material::Material;
use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone, Debug)]
pub struct Sphere {
    center: Point3,
    radius: FloatType,
    material: Box<dyn Material>,
}

#[derive(Clone, Debug)]
pub struct MovingSphere {
    center0: (Point3, FloatType),
    center1: (Point3, FloatType),
    radius: FloatType,
    material: Box<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: FloatType, material: Box<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl MovingSphere {
    pub fn new(center0: (Point3, FloatType), center1: (Point3, FloatType), radius: FloatType, material: Box<dyn Material>) -> Self {
        Self {
            center0,
            center1,
            radius,
            material,
        }
    }

    fn center(&self, t: FloatType) -> Point3 {
        self.center0.0 + ((t - self.center0.1) / (self.center1.1 - self.center0.1)) * (self.center1.0 - self.center0.0)
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

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        BoundingBox::new(
            self.center - vec3(self.radius, self.radius, self.radius),
            self.center + vec3(self.radius, self.radius, self.radius),
        )
    }
}

impl Hittable for MovingSphere {
    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<HitResult> {
        let center = self.center(ray.time);
        let oc = ray.origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - center) / self.radius;
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
                let outward_normal = (hit_point - center) / self.radius;
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

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let box0 = BoundingBox::new(
            self.center(t0) - vec3(self.radius, self.radius, self.radius),
            self.center(t0) + vec3(self.radius, self.radius, self.radius),
        );
        let box1 = BoundingBox::new(
            self.center(t1) - vec3(self.radius, self.radius, self.radius),
            self.center(t1) + vec3(self.radius, self.radius, self.radius),
        );

        BoundingBox::surrounding_box(&box0, &box1)
    }
}