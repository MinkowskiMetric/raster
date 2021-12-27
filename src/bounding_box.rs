use std::iter::FromIterator;

use crate::{math::*, Ray, Transformable};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pt_min: Point3,
    pt_max: Point3,
}

pub struct BoundingBoxIntersectionTester {
    ray_origin: Point3,
    ray_inv_dir_x: FloatType,
    ray_inv_dir_y: FloatType,
    ray_inv_dir_z: FloatType,
}

impl BoundingBoxIntersectionTester {
    pub fn new(ray: &Ray) -> Self {
        Self {
            ray_origin: ray.origin,
            ray_inv_dir_x: 1.0 / ray.direction.x,
            ray_inv_dir_y: 1.0 / ray.direction.y,
            ray_inv_dir_z: 1.0 / ray.direction.z,
        }
    }

    pub fn intersect(
        &self,
        bounding_box: &BoundingBox,
        mut t_min: FloatType,
        mut t_max: FloatType,
    ) -> bool {
        let (t0, t1) = if self.ray_inv_dir_x < 0.0 {
            (
                (bounding_box.pt_max.x - self.ray_origin.x) * self.ray_inv_dir_x,
                (bounding_box.pt_min.x - self.ray_origin.x) * self.ray_inv_dir_x,
            )
        } else {
            (
                (bounding_box.pt_min.x - self.ray_origin.x) * self.ray_inv_dir_x,
                (bounding_box.pt_max.x - self.ray_origin.x) * self.ray_inv_dir_x,
            )
        };

        t_min = t0.max(t_min);
        t_max = t1.min(t_max);

        if t_min > t_max {
            return false;
        }

        let (t0, t1) = if self.ray_inv_dir_y < 0.0 {
            (
                (bounding_box.pt_max.y - self.ray_origin.y) * self.ray_inv_dir_y,
                (bounding_box.pt_min.y - self.ray_origin.y) * self.ray_inv_dir_y,
            )
        } else {
            (
                (bounding_box.pt_min.y - self.ray_origin.y) * self.ray_inv_dir_y,
                (bounding_box.pt_max.y - self.ray_origin.y) * self.ray_inv_dir_y,
            )
        };

        t_min = t0.max(t_min);
        t_max = t1.min(t_max);

        if t_min > t_max {
            return false;
        }

        let (t0, t1) = if self.ray_inv_dir_z < 0.0 {
            (
                (bounding_box.pt_max.z - self.ray_origin.z) * self.ray_inv_dir_z,
                (bounding_box.pt_min.z - self.ray_origin.z) * self.ray_inv_dir_z,
            )
        } else {
            (
                (bounding_box.pt_min.z - self.ray_origin.z) * self.ray_inv_dir_z,
                (bounding_box.pt_max.z - self.ray_origin.z) * self.ray_inv_dir_z,
            )
        };

        t_min = t0.max(t_min);
        t_max = t1.min(t_max);

        t_max > t_min
    }
}

impl Transformable for BoundingBox {
    type Target = BoundingBox;

    fn core_transform(self, transform: &Matrix4, _inverse_transform: &Matrix4) -> Self::Target {
        self.all_corners()
            .iter()
            .map(|p| transform.transform_point(*p))
            .collect()
    }
}

/// # Safety
///
/// only call this if the CPU supports AVX
/*#[inline]
#[target_feature(enable = "avx")]
unsafe fn max_v(v: std::arch::x86_64::__m256d) -> std::arch::x86_64::__m128d {
    use std::arch::x86_64::*;

    let x = _mm256_extractf128_pd(v, 0); // extract v[0], and v[1]
    let y = _mm256_extractf128_pd(v, 1); // extract v[2], and v[3]
    let m1 = _mm_max_pd(x, y); // m1[0] = max(v[0], v[2]), m1[1] = max(v[1], v[3])
    let m2 = _mm_permute_pd(m1, 1); // m2[0] = m1[1], m2[1] = m1[0]
    _mm_max_pd(m1, m2)
}

/// # Safety
///
/// only call this if the CPU supports AVX
#[inline]
#[target_feature(enable = "avx")]
unsafe fn min_v(v: std::arch::x86_64::__m256d) -> std::arch::x86_64::__m128d {
    use std::arch::x86_64::*;

    let x = _mm256_extractf128_pd(v, 0); // extract v[0], and v[1]
    let y = _mm256_extractf128_pd(v, 1); // extract v[2], and v[3]
    let m1 = _mm_min_pd(x, y); // m1[0] = min(v[0], v[2]), m1[1] = min(v[1], v[3])
    let m2 = _mm_permute_pd(m1, 1); // m2[0] = m1[1], m2[1] = m1[0]
    _mm_min_pd(m1, m2)
}*/

impl BoundingBox {
    pub fn new(pt1: Point3, pt2: Point3) -> Self {
        BoundingBox {
            pt_min: Point3::new(pt1.x.min(pt2.x), pt1.y.min(pt2.y), pt1.z.min(pt2.z)),
            pt_max: Point3::new(pt1.x.max(pt2.x), pt1.y.max(pt2.y), pt1.z.max(pt2.z)),
        }
    }

    pub fn empty_box() -> Self {
        let zero_point = Point3::new(0.0, 0.0, 0.0);
        BoundingBox {
            pt_min: zero_point,
            pt_max: zero_point,
        }
    }

    pub fn containing_point(pt: Point3) -> Self {
        let epsilon_offset: Vector3 = vec3(0.0001, 0.0001, 0.0001);
        Self {
            pt_min: pt - epsilon_offset,
            pt_max: pt + epsilon_offset,
        }
    }

    pub fn combine(self, other: Self) -> Self {
        Self::surrounding_box(&self, &other)
    }

    pub fn surrounding_box(box0: &BoundingBox, box1: &BoundingBox) -> Self {
        let pt_min = Point3::new(
            box0.min_point().x.min(box1.min_point().x),
            box0.min_point().y.min(box1.min_point().y),
            box0.min_point().z.min(box1.min_point().z),
        );
        let pt_max = Point3::new(
            box0.max_point().x.max(box1.max_point().x),
            box0.max_point().y.max(box1.max_point().y),
            box0.max_point().z.max(box1.max_point().z),
        );
        Self::new(pt_min, pt_max)
    }

    pub fn min_point(&self) -> &Point3 {
        &self.pt_min
    }

    pub fn max_point(&self) -> &Point3 {
        &self.pt_max
    }

    pub fn all_corners(&self) -> [Point3; 8] {
        [
            Point3::new(self.min_point().x, self.min_point().y, self.min_point().z),
            Point3::new(self.min_point().x, self.min_point().y, self.max_point().z),
            Point3::new(self.min_point().x, self.max_point().y, self.min_point().z),
            Point3::new(self.min_point().x, self.max_point().y, self.max_point().z),
            Point3::new(self.max_point().x, self.min_point().y, self.min_point().z),
            Point3::new(self.max_point().x, self.min_point().y, self.max_point().z),
            Point3::new(self.max_point().x, self.max_point().y, self.min_point().z),
            Point3::new(self.max_point().x, self.max_point().y, self.max_point().z),
        ]
    }

    pub fn intersect_with_tester(
        &self,
        tester: &BoundingBoxIntersectionTester,
        t_min: FloatType,
        t_max: FloatType,
    ) -> bool {
        tester.intersect(self, t_min, t_max)
    }

    pub fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> bool {
        self.intersect_with_tester(&BoundingBoxIntersectionTester::new(ray), t_min, t_max)
    }

    /*/// # Safety
    ///
    /// only call this if the CPU supports AVX
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

        0 != _mm_comilt_sd(t_min, t_max)
    }*/
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty_box()
    }
}

impl FromIterator<Point3> for BoundingBox {
    fn from_iter<T: IntoIterator<Item = Point3>>(iter: T) -> Self {
        iter.into_iter()
            .map(BoundingBox::containing_point)
            .collect()
    }
}

impl FromIterator<BoundingBox> for BoundingBox {
    fn from_iter<T: IntoIterator<Item = BoundingBox>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        iter.next()
            .map(|first| iter.fold(first, |l, r| l.combine(r)))
            .unwrap_or_default()
    }
}
