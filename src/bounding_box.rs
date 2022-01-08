use std::{
    iter::FromIterator,
    simd::{f32x4, mask32x4, simd_swizzle, Simd, Which::*},
};

use crate::{math::*, Ray, Transformable};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pt_min: f32x4,
    pt_max: f32x4,
}

pub struct BoundingBoxIntersectionTester {
    ray_origin: f32x4,
    ray_inv_dir: f32x4,
    ray_dir_sign: mask32x4,
}

impl BoundingBoxIntersectionTester {
    pub fn new(ray: &Ray) -> Self {
        let ray_origin = ray.origin_v();
        let ray_inv_dir = ray.direction_v().recip();
        let ray_dir_sign = ray_inv_dir.is_sign_positive();

        Self {
            ray_origin,
            ray_inv_dir,
            ray_dir_sign,
        }
    }

    pub fn intersect(
        &self,
        bounding_box: &BoundingBox,
        t_min: FloatType,
        t_max: FloatType,
    ) -> bool {
        let t0 = simd_swizzle!(
            (bounding_box.pt_min - self.ray_origin) * self.ray_inv_dir,
            Simd::splat(t_min),
            [First(0), First(1), First(2), Second(0)]
        );
        let t1 = simd_swizzle!(
            (bounding_box.pt_max - self.ray_origin) * self.ray_inv_dir,
            Simd::splat(t_max),
            [First(0), First(1), First(2), Second(0)]
        );

        let t_min = self.ray_dir_sign.select(t0, t1);
        let t_max = self.ray_dir_sign.select(t1, t0);

        let t_min = t_min.horizontal_max();
        let t_max = t_max.horizontal_min();

        t_min < t_max
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

impl BoundingBox {
    pub fn new(pt1: Point3, pt2: Point3) -> Self {
        let pt1 = f32x4::from_array([pt1.x, pt1.y, pt1.z, 1.0]);
        let pt2 = f32x4::from_array([pt2.x, pt2.y, pt2.z, 1.0]);

        let pt_max = pt1.max(pt2);
        let pt_min = pt1.min(pt2);

        BoundingBox { pt_min, pt_max }
    }

    pub fn empty_box() -> Self {
        let zero_point = Point3::new(0.0, 0.0, 0.0);
        Self::new(zero_point, zero_point)
    }

    pub fn containing_point(pt: Point3) -> Self {
        let epsilon_offset: Vector3 = vec3(0.0001, 0.0001, 0.0001);
        Self::new(pt - epsilon_offset, pt + epsilon_offset)
    }

    #[must_use]
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

    pub fn min_point(&self) -> Point3 {
        let pt_min_arr = self.pt_min.to_array();
        point3(pt_min_arr[0], pt_min_arr[1], pt_min_arr[2])
    }

    pub fn max_point(&self) -> Point3 {
        let pt_max_arr = self.pt_max.to_array();
        point3(pt_max_arr[0], pt_max_arr[1], pt_max_arr[2])
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
