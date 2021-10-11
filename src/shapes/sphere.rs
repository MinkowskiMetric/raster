use super::{Primitive, PrimitiveHitResult, TransformablePrimitive, UntransformedPrimitive};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

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
}

impl Sphere {
    pub fn new(center: Point3, radius: FloatType) -> Self {
        Self { center, radius }
    }

    /*/// # Safety
    ///
    /// only call this if the CPU supports AVX
    #[inline]
    #[target_feature(enable = "avx")]
    unsafe fn intersect_avx(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
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
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
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
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }
        }

        None
    }*/
}

impl Primitive for Sphere {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        stats.count_sphere_test();
        let ray_origin = ray.origin;
        let oc = ray_origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0)
                    .cross(hit_point - self.center)
                    .normalize();
                let front_face = ray.direction.dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - self.center) / self.radius;
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - self.center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0)
                    .cross(hit_point - self.center)
                    .normalize();
                let front_face = ray.direction.dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - self.center) / self.radius;
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }
        }

        None
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        BoundingBox::new(
            self.center - vec3(self.radius, self.radius, self.radius),
            self.center + vec3(self.radius, self.radius, self.radius),
        )
    }
}

impl UntransformedPrimitive for Sphere {}

#[derive(Clone, Debug)]
pub struct MovingSphere {
    space_origin: Point3,
    time_origin: FloatType,
    velocity: Vector3,
    radius: FloatType,
}

impl MovingSphere {
    pub fn new(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
    ) -> Self {
        let space_origin = center0.0;
        let time_origin = center0.1;
        let velocity = (center1.0 - center0.0) / (center1.1 - center0.1);
        Self {
            space_origin,
            time_origin,
            velocity,
            radius,
        }
    }

    /// # Safety
    ///
    /// only call this if the CPU supports AVX
    /*#[inline]
    #[target_feature(enable = "avx")]
    unsafe fn center_v(&self, t: FloatType) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;

        let space_origin = self.space_origin.load_v();
        let time_origin = self.time_origin.load_v();
        let velocity = self.velocity.load_v();

        let t_shift = _mm256_sub_pd(_mm256_set1_pd(t), time_origin);
        let translate = _mm256_mul_pd(velocity, t_shift);

        _mm256_add_pd(space_origin, translate)
    }*/

    fn center(&self, t: FloatType) -> Point3 {
        let t_shift = t - self.time_origin;
        let translate = self.velocity * t_shift;

        self.space_origin + translate
    }

    /*/// # Safety
    ///
    /// only call this if the CPU supports AVX
    #[inline]
    #[target_feature(enable = "avx")]
    unsafe fn intersect_avx(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
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
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
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
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }
        }

        None
    }*/
}

impl Primitive for MovingSphere {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        stats.count_moving_sphere_test();
        let center = self.center(ray.time);
        let ray_origin = ray.origin;
        let oc = ray_origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);
        let discriminant = (b * b) - (a * c);
        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }

            let temp = (-b + discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let hit_point = ray.origin + (temp * ray.direction);
                let outward_normal = (hit_point - center) / self.radius;
                let tangent = vec3(0.0, 1.0, 0.0).cross(hit_point - center).normalize();
                let front_face = ray.direction.dot(outward_normal) < 0.0;

                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                let bitangent = outward_normal.cross(tangent).normalize();

                let normalized_hitpoint = (hit_point - center) / self.radius;
                let uv = get_sphere_uv(normalized_hitpoint);
                return Some(PrimitiveHitResult::new(
                    temp,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                ));
            }
        }

        None
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

impl UntransformedPrimitive for MovingSphere {}

pub mod factories {
    use super::*;

    pub fn unit_sphere() -> <Sphere as TransformablePrimitive>::Target {
        Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0).identity()
    }

    pub fn sphere(center: Point3, radius: FloatType) -> <Sphere as TransformablePrimitive>::Target {
        unit_sphere()
            .scale(radius)
            .translate(center - Point3::new(0.0, 0.0, 0.0))
    }

    pub fn moving_sphere(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
    ) -> MovingSphere {
        MovingSphere::new(center0, center1, radius)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::factories::*;

    #[test]
    fn test_sphere_normals() {
        let sp = sphere(Point3::new(0.0, 0.0, 0.0), 1.0);

        let mut stats = crate::TracingStats::new();
        let result = sp.intersect(
            &Ray::new(Point3::new(0.0, 0.0, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
            &mut stats,
        );
        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!(result.distance(), 9.0);
        assert_eq!(result.hit_point(), Point3::new(0.0, 0.0, -1.0));
        assert_eq!(result.surface_normal(), vec3(0.0, 0.0, -1.0));
        assert_eq!(result.tangent(), vec3(-1.0, 0.0, 0.0));
        assert_eq!(result.bitangent(), vec3(0.0, 1.0, 0.0));
    }
}
