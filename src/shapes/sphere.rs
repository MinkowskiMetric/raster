use super::{CoreHittable, HitResult};
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
                let hit_point = _mm256_add_pd(
                    ray_origin,
                    _mm256_mul_pd(_mm256_set1_pd(temp), ray_direction),
                );
                let outward_normal = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let front_face = _mm256_dot_pd(ray_direction, outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    _mm256_mul_pd(outward_normal, _mm256_set1_pd(-1.0))
                };

                let normalized_hitpoint = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let normalized_hitpoint = M256Vector3::from_v(normalized_hitpoint).into();
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: M256Point3::from_v(hit_point).into(),
                    surface_normal: M256Vector3::from_v(surface_normal).into(),
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = _mm256_add_pd(
                    ray_origin,
                    _mm256_mul_pd(_mm256_set1_pd(temp), ray_direction),
                );
                let outward_normal = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let front_face = _mm256_dot_pd(ray_direction, outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    _mm256_mul_pd(outward_normal, _mm256_set1_pd(-1.0))
                };

                let normalized_hitpoint = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let normalized_hitpoint = M256Vector3::from_v(normalized_hitpoint).into();
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: M256Point3::from_v(hit_point).into(),
                    surface_normal: M256Vector3::from_v(surface_normal).into(),
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

impl<T: 'static + Material + Clone> CoreHittable for Sphere<T> {
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
                let hit_point = _mm256_add_pd(
                    ray_origin,
                    _mm256_mul_pd(_mm256_set1_pd(temp), ray_direction),
                );
                let outward_normal = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let front_face = _mm256_dot_pd(ray_direction, outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    _mm256_mul_pd(outward_normal, _mm256_set1_pd(-1.0))
                };

                let normalized_hitpoint = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let normalized_hitpoint = M256Vector3::from_v(normalized_hitpoint).into();
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: M256Point3::from_v(hit_point).into(),
                    surface_normal: M256Vector3::from_v(surface_normal).into(),
                    front_face,
                    material: &self.material,
                    u,
                    v,
                });
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = _mm256_add_pd(
                    ray_origin,
                    _mm256_mul_pd(_mm256_set1_pd(temp), ray_direction),
                );
                let outward_normal = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let front_face = _mm256_dot_pd(ray_direction, outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    _mm256_mul_pd(outward_normal, _mm256_set1_pd(-1.0))
                };

                let normalized_hitpoint = _mm256_div_pd(
                    _mm256_sub_pd(hit_point, center),
                    _mm256_set1_pd(self.radius),
                );
                let normalized_hitpoint = M256Vector3::from_v(normalized_hitpoint).into();
                let (u, v) = get_sphere_uv(normalized_hitpoint);
                return Some(HitResult {
                    distance: temp,
                    hit_point: M256Point3::from_v(hit_point).into(),
                    surface_normal: M256Vector3::from_v(surface_normal).into(),
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

impl<T: 'static + Material + Clone> CoreHittable for MovingSphere<T> {
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
