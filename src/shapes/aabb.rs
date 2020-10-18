use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pt_min: M256Point3,
    pt_max: M256Point3,
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

    pub fn all_corners(&self) -> [Point3; 8] {
        [
            Point3::new(
                self.min_point().into_point().x,
                self.min_point().into_point().y,
                self.min_point().into_point().z,
            ),
            Point3::new(
                self.min_point().into_point().x,
                self.min_point().into_point().y,
                self.max_point().into_point().z,
            ),
            Point3::new(
                self.min_point().into_point().x,
                self.max_point().into_point().y,
                self.min_point().into_point().z,
            ),
            Point3::new(
                self.min_point().into_point().x,
                self.max_point().into_point().y,
                self.max_point().into_point().z,
            ),
            Point3::new(
                self.max_point().into_point().x,
                self.min_point().into_point().y,
                self.min_point().into_point().z,
            ),
            Point3::new(
                self.max_point().into_point().x,
                self.min_point().into_point().y,
                self.max_point().into_point().z,
            ),
            Point3::new(
                self.max_point().into_point().x,
                self.max_point().into_point().y,
                self.min_point().into_point().z,
            ),
            Point3::new(
                self.max_point().into_point().x,
                self.max_point().into_point().y,
                self.max_point().into_point().z,
            ),
        ]
    }

    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn intersect_avx(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> bool {
        use std::arch::x86_64::*;

        // We can probably do better than this if we improve the way these are stored
        let ray_origin = ray.origin.load_v();
        let ray_inv_direction = ray.inv_direction.load_v();
        let dir_sign = _mm256_cmp_pd(ray_inv_direction, _mm256_setzero_pd(), _CMP_LT_OQ);
        let pt_min = self.pt_min.load_v();
        let pt_max = self.pt_max.load_v();

        // Add t_min and t_max into the fourth value in the vector
        let t0 = _mm256_mul_pd(_mm256_sub_pd(pt_min, ray_origin), ray_inv_direction);
        let t0 = _mm256_blend_pd(_mm256_set1_pd(t_min), t0, 0xe);
        let t1 = _mm256_mul_pd(_mm256_sub_pd(pt_max, ray_origin), ray_inv_direction);
        let t1 = _mm256_blend_pd(_mm256_set1_pd(t_max), t1, 0xe);

        // Swizzle the values to get the min and max values the right way round according to
        // the direction and select the min and max values
        let t_min_l = _mm256_and_pd(dir_sign, t1);
        let t_min_r = _mm256_andnot_pd(dir_sign, t0);
        let t_min_f = _mm256_or_pd(t_min_l, t_min_r);
        let t_min = max_v(t_min_f);
        let t_max_l = _mm256_and_pd(dir_sign, t0);
        let t_max_r = _mm256_andnot_pd(dir_sign, t1);
        let t_max_f = _mm256_or_pd(t_max_l, t_max_r);
        let t_max = min_v(t_max_f);

        let cmp = 0 != _mm_comilt_sd(t_min, t_max);

        cmp
    }
}
