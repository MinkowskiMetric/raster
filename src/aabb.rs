use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pt_min: M256Point3,
    pt_max: M256Point3,
}

#[cfg(not(all(target_feature = "avx", not(feature = "disableavx"))))]
fn test_axis(
    pt_min: FloatType,
    pt_max: FloatType,
    ray_origin: FloatType,
    ray_direction: FloatType,
    t_min: &mut FloatType,
    t_max: &mut FloatType,
) -> bool {
    let inverse_direction = 1.0 / ray_direction;
    let mut t0 = (pt_min - ray_origin) * inverse_direction;
    let mut t1 = (pt_max - ray_origin) * inverse_direction;

    if inverse_direction < 0.0 {
        std::mem::swap(&mut t0, &mut t1);
    }

    *t_min = t0.max(*t_min);
    *t_max = t1.min(*t_max);

    !(t_max <= t_min)
}

#[inline]
#[target_feature(enable = "avx")]
unsafe fn max_v(v: std::arch::x86_64::__m256d) -> std::arch::x86_64::__m128d {
    use std::arch::x86_64::*;

    let x = _mm256_extractf128_pd(v, 0); // extract v[0], and v[1]
    let y = _mm256_extractf128_pd(v, 1); // extract v[2], and v[3]
    let m1 = _mm_max_pd(x, y); // m1[0] = max(v[0], v[2]), m1[1] = max(v[1], v[3])
    let m2 = _mm_permute_pd(m1, 1); // m2[0] = m1[1], m2[1] = m1[0]
    let m = _mm_max_pd(m1, m2);

    m
}

#[inline]
#[target_feature(enable = "avx")]
unsafe fn min_v(v: std::arch::x86_64::__m256d) -> std::arch::x86_64::__m128d {
    use std::arch::x86_64::*;

    let x = _mm256_extractf128_pd(v, 0); // extract v[0], and v[1]
    let y = _mm256_extractf128_pd(v, 1); // extract v[2], and v[3]
    let m1 = _mm_min_pd(x, y); // m1[0] = min(v[0], v[2]), m1[1] = min(v[1], v[3])
    let m2 = _mm_permute_pd(m1, 1); // m2[0] = m1[1], m2[1] = m1[0]
    let m = _mm_min_pd(m1, m2);

    m
}

impl BoundingBox {
    pub fn new(pt1: Point3, pt2: Point3) -> Self {
        BoundingBox {
            pt_min: Point3::new(pt1.x.min(pt2.x), pt1.y.min(pt2.y), pt1.z.min(pt2.z)).into(),
            pt_max: Point3::new(pt1.x.max(pt2.x), pt1.y.max(pt2.y), pt1.z.max(pt2.z)).into(),
        }
    }

    pub fn empty_box() -> Self {
        let zero_point = Point3::new(0.0, 0.0, 0.0);
        BoundingBox {
            pt_min: zero_point.into(),
            pt_max: zero_point.into(),
        }
    }

    pub fn surrounding_box(box0: &BoundingBox, box1: &BoundingBox) -> Self {
        let pt_min = Point3::new(
            box0.min_point().x().min(box1.min_point().x()),
            box0.min_point().y().min(box1.min_point().y()),
            box0.min_point().z().min(box1.min_point().z()),
        );
        let pt_max = Point3::new(
            box0.max_point().x().max(box1.max_point().x()),
            box0.max_point().y().max(box1.max_point().y()),
            box0.max_point().z().max(box1.max_point().z()),
        );
        Self::new(pt_min, pt_max)
    }

    pub fn min_point(&self) -> &M256Point3 {
        &self.pt_min
    }

    pub fn max_point(&self) -> &M256Point3 {
        &self.pt_max
    }

    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn intersect_avx(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> bool {
        use std::arch::x86_64::*;

        // We can probably do better than this if we improve the way these are stored
        let ray_origin = ray.origin.load_v();
        let ray_inv_direction = ray.inv_direction.load_v();
        let dir_sign = ray.sign.load_v();
        let pt_min = self.pt_min.load_v();
        let pt_max = self.pt_max.load_v();

        // Add t_min and t_max into the fourth value in the vector
        let t0 = _mm256_mul_pd(_mm256_sub_pd(pt_min, ray_origin), ray_inv_direction);
        let t0 = _mm256_blend_pd(_mm256_set1_pd(t_min), t0, 0xe);
        let t1 = _mm256_mul_pd(_mm256_sub_pd(pt_max, ray_origin), ray_inv_direction);
        let t1 = _mm256_blend_pd(_mm256_set1_pd(t_max), t1, 0xe);

        // Swizzle the values to get the min and max values the right way round according to
        // the direction and select the min and max values
        let t_min = max_v(_mm256_blendv_pd(t0, t1, dir_sign));
        let t_max = min_v(_mm256_blendv_pd(t1, t0, dir_sign));

        0 != _mm_comilt_sd(t_min, t_max)
    }

    #[inline]
    pub fn intersect(&self, ray: &Ray, mut t_min: FloatType, mut t_max: FloatType) -> bool {
        if is_x86_feature_detected!("avx") {
            unsafe { self.intersect_avx(ray, t_min, t_max) }
        } else {
            test_axis(
                self.pt_min.x(),
                self.pt_max.x(),
                ray.origin.x(),
                ray.direction.x,
                &mut t_min,
                &mut t_max,
            ) && test_axis(
                self.pt_min.y(),
                self.pt_max.y(),
                ray.origin.y(),
                ray.direction.y,
                &mut t_min,
                &mut t_max,
            ) && test_axis(
                self.pt_min.z(),
                self.pt_max.z(),
                ray.origin.z(),
                ray.direction.z,
                &mut t_min,
                &mut t_max,
            )
        }
    }
}
