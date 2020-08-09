use crate::aabb::BoundingBox;
use crate::hittable::{HitResult, Hittable};
use crate::material::SharedMaterial;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::stats::TracingStats;

fn get_sphere_uv(p: Vector3) -> (FloatType, FloatType) {
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();

    let u = 1.0 - (phi + constants::PI) / (2.0 * constants::PI);
    let v = (theta + constants::PI / 2.0) / constants::PI;

    (u, v)
}

#[derive(Clone, Debug)]
pub struct Sphere {
    center: Point3,
    radius: FloatType,
    material: SharedMaterial,
}

#[derive(Clone, Debug)]
pub struct MovingSphere {
    center0: (Point3, FloatType),
    center1: (Point3, FloatType),
    radius: FloatType,
    material: SharedMaterial,
}

impl Sphere {
    pub fn new(center: Point3, radius: FloatType, material: SharedMaterial) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl MovingSphere {
    pub fn new(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
        material: SharedMaterial,
    ) -> Self {
        Self {
            center0,
            center1,
            radius,
            material,
        }
    }

    fn center(&self, t: FloatType) -> Point3 {
        self.center0.0
            + ((t - self.center0.1) / (self.center1.1 - self.center0.1))
                * (self.center1.0 - self.center0.0)
    }
}

impl Hittable for Sphere {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult> {
        stats.count_sphere_test();
        let ray_origin = ray.origin.into_point();
        let oc = ray_origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray_origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let (u, v) = get_sphere_uv((hit_point - self.center) / self.radius);
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray_origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let (u, v) = get_sphere_uv((hit_point - self.center) / self.radius);
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                    u,
                    v,
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
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult> {
        stats.count_moving_sphere_test();
        let center = self.center(ray.time);
        let ray_origin = ray.origin.into_point();
        let oc = ray_origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray_origin + (temp * ray.direction);
                let outward_normal = (hit_point - center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let (u, v) = get_sphere_uv((hit_point - center) / self.radius);
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray_origin + (temp * ray.direction);
                let outward_normal = (hit_point - center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let (u, v) = get_sphere_uv((hit_point - center) / self.radius);
                return Some(HitResult {
                    distance: temp,
                    hit_point,
                    surface_normal,
                    front_face,
                    material: &self.material,
                    u,
                    v,
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
