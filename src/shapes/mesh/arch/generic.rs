pub mod mesh {
    use crate::{math::*, Bounded, BoundingBox, Ray};

    pub const TRIANGLE_MESH_GROUP_SIZE: usize = 128;

    #[derive(Debug, Clone)]
    pub struct IntersectTriangle {
        v0: Point3,
        v0v1: Vector3,
        v0v2: Vector3,
        start_index: usize,
    }

    impl IntersectTriangle {
        pub fn new(start_index: usize, v0: Point3, v1: Point3, v2: Point3) -> Self {
            let v0v1 = v1 - v0;
            let v0v2 = v2 - v0;

            Self {
                v0,
                v0v1,
                v0v2,
                start_index,
            }
        }
    }

    pub struct IntersectTriangleHitResult {
        pub start_index: usize,
        pub t: FloatType,
        pub u: FloatType,
        pub v: FloatType,
    }

    impl Bounded for IntersectTriangle {
        fn bounding_box(&self) -> BoundingBox {
            [self.v0, self.v0 + self.v0v1, self.v0 + self.v0v2]
                .iter()
                .cloned()
                .map(BoundingBox::containing_point)
                .collect()
        }
    }

    // We want to eventually use SIMD to do the triangle intersections, which means that
    // we need to operate on slices of triangles to be efficient, so this is what we do
    pub fn intersect_triangle_slice(
        triangles: &[IntersectTriangle],
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<IntersectTriangleHitResult> {
        triangles
            .iter()
            .filter_map(|triangle| {
                let pvec = ray.direction.cross(triangle.v0v2);
                let det = triangle.v0v1.dot(pvec);

                if det.abs() < constants::EPSILON {
                    return None;
                }

                let inv_det = 1.0 / det;
                let tvec = ray.origin - triangle.v0;
                let u = tvec.dot(pvec) * inv_det;
                if !(0.0..=1.0).contains(&u) {
                    return None;
                }

                let qvec = tvec.cross(triangle.v0v1);
                let v = ray.direction.dot(qvec) * inv_det;
                if v < 0.0 || u + v > 1.0 {
                    return None;
                }

                let t = triangle.v0v2.dot(qvec) * inv_det;
                if !(t_min..=t_max).contains(&t) {
                    return None;
                }

                Some(IntersectTriangleHitResult {
                    t,
                    u,
                    v,
                    start_index: triangle.start_index,
                })
            })
            .min_by(|l, r| l.t.partial_cmp(&r.t).unwrap())
    }
}
