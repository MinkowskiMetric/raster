use super::{HitResult, Shape, SimpleShape, UntransformedShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::RenderStatsCollector;
use crate::{BoundingBox, Material};

fn get_sphere_uv(p: Vector3) -> (FloatType, FloatType) {
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();

    let u = 1.0 - (phi + constants::PI) / (2.0 * constants::PI);
    let v = (theta + constants::PI / 2.0) / constants::PI;

    (u, v)
}

#[derive(Clone, Debug)]
pub struct Sphere<T: 'static + Material + Clone> {
    center: M256Point3,
    radius: FloatType,
    material: T,
}

impl<T: 'static + Material + Clone> Sphere<T> {
    pub fn new(center: Point3, radius: FloatType, material: T) -> Self {
        Self {
            center: center.into(),
            radius,
            material,
        }
    }

    #[inline]
    #[target_feature(enable = "avx")]
    unsafe fn intersect_avx(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult> {
        use std::arch::x86_64::*;

        stats.count_sphere_test();
        let ray_origin = ray.origin.load_v();
        let ray_direction = ray.direction.load_v();
        let center = self.center.load_v();
        let oc = _mm256_sub_pd(ray_origin, center);

        // Doing a dot product in AVX is a pain in the bum. But... we need to do three. We can make it less
        // of a pain if we do all three at the same time.
        let rdrd = _mm256_mul_pd(ray_direction, ray_direction);
        let ocrd = _mm256_mul_pd(oc, ray_direction);
        let ococ = _mm256_mul_pd(oc, oc);
        let dcdc = ococ; // Space for a fourth dot product if we needed one

        let tmp_01 = _mm256_hadd_pd(dcdc, ococ); // [ dcdc[0] + dcdc[1], ococ[0] + ococ[1], dcdc[2] + dcdc[3], ococ[2] + ococ[3] ]
        let tmp_23 = _mm256_hadd_pd(ocrd, rdrd); // [ ocrd[0] + ocrd[1], rdrd[0] + rdrd[1], ocrd[2] + ocrd[3], rdrd[2] + rdrd[3] ]

        let swapped = _mm256_permute2f128_pd(tmp_01, tmp_23, 0x21);
        // [ dcdc[2] + dcdc[3], ococ[2] + ococ[3], ocrd[0] + ocrd[1], rdrd[0] + rdrd[1] ]
        let blended = _mm256_blend_pd(tmp_01, tmp_23, 0xc);
        // [ dcdc[0] + dcdc[1], ococ[0] + ococ[1], ocrd[2] + ocrd[3], rdrd[2] + rdrd[3] ]

        let abcd = _mm256_add_pd(swapped, blended); // [ rd dot rd, oc dot rd, oc dot oc, NOTHING ]

        let abcd = M256Vector3::from_v(abcd).into_vector();
        let a = abcd.x;
        let b = abcd.y;
        let c = abcd.z - (self.radius * self.radius);

        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let center: Point3 = M256Point3::from_v(center).into();
                let hit_point = ray.origin.into_point() + (temp * ray.direction.into_vector());
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.into_vector().dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: hit_point.into(),
                    surface_normal: surface_normal.into(),
                    tangent,
                    bitangent,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let center: Point3 = M256Point3::from_v(center).into();
                let hit_point = ray.origin.into_point() + (temp * ray.direction.into_vector());
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.into_vector().dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: hit_point.into(),
                    surface_normal: surface_normal.into(),
                    tangent,
                    bitangent,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }
        }

        return None;
    }
}

impl<T: 'static + Material + Clone> Shape for Sphere<T> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult> {
        unsafe { self.intersect_avx(ray, t_min, t_max, stats) }
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        BoundingBox::new(
            self.center.into_point() - vec3(self.radius, self.radius, self.radius),
            self.center.into_point() + vec3(self.radius, self.radius, self.radius),
        )
    }
}

impl<T: 'static + Material + Clone> SimpleShape for Sphere<T> {}
impl<T: 'static + Material + Clone> UntransformedShape for Sphere<T> {}

#[derive(Clone, Debug)]
pub struct MovingSphere<T: 'static + Material + Clone> {
    space_origin: M256Point3,
    time_origin: M256Point3,
    velocity: M256Vector3,
    radius: FloatType,
    material: T,
}

impl<T: 'static + Material + Clone> MovingSphere<T> {
    pub fn new(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
        material: T,
    ) -> Self {
        let space_origin = center0.0.into();
        let time_origin = Point3::new(center0.1, center0.1, center0.1).into();
        let velocity = ((center1.0 - center0.0) / (center1.1 - center0.1)).into();
        Self {
            space_origin,
            time_origin,
            velocity,
            radius,
            material,
        }
    }

    #[inline]
    #[target_feature(enable = "avx")]
    unsafe fn center_v(&self, t: FloatType) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;

        let space_origin = self.space_origin.load_v();
        let time_origin = self.time_origin.load_v();
        let velocity = self.velocity.load_v();

        let t_shift = _mm256_sub_pd(_mm256_set1_pd(t), time_origin);
        let translate = _mm256_mul_pd(velocity, t_shift);

        let center = _mm256_add_pd(space_origin, translate);

        center
    }

    fn center(&self, t: FloatType) -> M256Point3 {
        unsafe { M256Point3::from_v(self.center_v(t)) }
    }

    #[inline]
    #[target_feature(enable = "avx")]
    unsafe fn intersect_avx(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult> {
        use std::arch::x86_64::*;

        stats.count_moving_sphere_test();
        let ray_origin = ray.origin.load_v();
        let ray_direction = ray.direction.load_v();
        let center = self.center_v(ray.time);
        let oc = _mm256_sub_pd(ray_origin, center);

        // Doing a dot product in AVX is a pain in the bum. But... we need to do three. We can make it less
        // of a pain if we do all three at the same time.
        let rdrd = _mm256_mul_pd(ray_direction, ray_direction);
        let ocrd = _mm256_mul_pd(oc, ray_direction);
        let ococ = _mm256_mul_pd(oc, oc);
        let dcdc = ococ; // Space for a fourth dot product if we needed one

        let tmp_01 = _mm256_hadd_pd(dcdc, ococ); // [ dcdc[0] + dcdc[1], ococ[0] + ococ[1], dcdc[2] + dcdc[3], ococ[2] + ococ[3] ]
        let tmp_23 = _mm256_hadd_pd(ocrd, rdrd); // [ ocrd[0] + ocrd[1], rdrd[0] + rdrd[1], ocrd[2] + ocrd[3], rdrd[2] + rdrd[3] ]

        let swapped = _mm256_permute2f128_pd(tmp_01, tmp_23, 0x21);
        // [ dcdc[2] + dcdc[3], ococ[2] + ococ[3], ocrd[0] + ocrd[1], rdrd[0] + rdrd[1] ]
        let blended = _mm256_blend_pd(tmp_01, tmp_23, 0xc);
        // [ dcdc[0] + dcdc[1], ococ[0] + ococ[1], ocrd[2] + ocrd[3], rdrd[2] + rdrd[3] ]

        let abcd = _mm256_add_pd(swapped, blended); // [ rd dot rd, oc dot rd, oc dot oc, NOTHING ]

        let abcd = M256Vector3::from_v(abcd).into_vector();
        let a = abcd.x;
        let b = abcd.y;
        let c = abcd.z - (self.radius * self.radius);

        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let center: Point3 = M256Point3::from_v(center).into();
                let hit_point = ray.origin.into_point() + (temp * ray.direction.into_vector());
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.into_vector().dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: hit_point.into(),
                    surface_normal: surface_normal.into(),
                    tangent,
                    bitangent,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let center: Point3 = M256Point3::from_v(center).into();
                let hit_point = ray.origin.into_point() + (temp * ray.direction.into_vector());
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.into_vector().dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: hit_point.into(),
                    surface_normal: surface_normal.into(),
                    tangent,
                    bitangent,
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }
        }

        return None;
    }
}

impl<T: 'static + Material + Clone> Shape for MovingSphere<T> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult> {
        unsafe { self.intersect_avx(ray, t_min, t_max, stats) }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let box0 = BoundingBox::new(
            self.center(t0).into_point() - vec3(self.radius, self.radius, self.radius),
            self.center(t0).into_point() + vec3(self.radius, self.radius, self.radius),
        );
        let box1 = BoundingBox::new(
            self.center(t1).into_point() - vec3(self.radius, self.radius, self.radius),
            self.center(t1).into_point() + vec3(self.radius, self.radius, self.radius),
        );

        BoundingBox::surrounding_box(&box0, &box1)
    }
}

impl<T: Material + 'static + Clone> SimpleShape for MovingSphere<T> {}
impl<T: 'static + Material + Clone> UntransformedShape for MovingSphere<T> {}

pub mod factories {
    use super::*;

    pub fn sphere<T: 'static + Material + Clone>(
        center: Point3,
        radius: FloatType,
        material: T,
    ) -> Sphere<T> {
        Sphere::new(center, radius, material)
    }

    pub fn moving_sphere<T: 'static + Material + Clone>(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
        material: T,
    ) -> MovingSphere<T> {
        MovingSphere::new(center0, center1, radius, material)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::factories::*;

    #[test]
    fn test_sphere_normals() {
        let sp = sphere(Point3::new(0.0, 0.0, 0.0), 1.0, dielectric(1.5));

        let mut stats = crate::TracingStats::new();
        let result = sp.intersect(
            &Ray::new(Point3::new(0.0, 0.0, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
            &mut stats,
        );
        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!(result.distance, 9.0);
        assert_eq!(result.hit_point, Point3::new(0.0, 0.0, -1.0));
        assert_eq!(result.surface_normal, vec3(0.0, 0.0, -1.0));
        assert_eq!(result.tangent, vec3(-1.0, 0.0, 0.0));
        assert_eq!(result.bitangent, vec3(0.0, 1.0, 0.0));
    }
}
