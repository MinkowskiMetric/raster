use crate::{math::*, DefaultSkinnable, Ray, Skinnable, Transformable};

pub trait IntersectResult {
    fn ray_origin(&self) -> Point3;
    fn distance(&self) -> FloatType;
    fn hit_point(&self) -> Point3;
    fn surface_normal(&self) -> Vector3;
    fn tangent(&self) -> Vector3;
    fn bitangent(&self) -> Vector3;
    fn front_face(&self) -> bool;
    fn uv(&self) -> Point2;
}

pub trait WrappedIntersectResult {
    type Wrapped: IntersectResult;

    fn intersect_result(&self) -> &Self::Wrapped;
}

impl<I: WrappedIntersectResult> IntersectResult for I {
    fn ray_origin(&self) -> Point3 {
        self.intersect_result().ray_origin()
    }

    fn distance(&self) -> FloatType {
        self.intersect_result().distance()
    }

    fn hit_point(&self) -> Point3 {
        self.intersect_result().hit_point()
    }

    fn surface_normal(&self) -> Vector3 {
        self.intersect_result().surface_normal()
    }

    fn tangent(&self) -> Vector3 {
        self.intersect_result().tangent()
    }

    fn bitangent(&self) -> Vector3 {
        self.intersect_result().bitangent()
    }

    fn front_face(&self) -> bool {
        self.intersect_result().front_face()
    }

    fn uv(&self) -> Point2 {
        self.intersect_result().uv()
    }
}

pub trait IntersectResultIteratorOps {
    type Result: IntersectResult;

    fn nearest(self) -> Option<Self::Result>;
}

impl<'a, I: IntersectResult, Iter: Iterator<Item = I>> IntersectResultIteratorOps for Iter {
    type Result = I;

    fn nearest(self) -> Option<Self::Result> {
        self.min_by(|xr, yr| {
            let xr_distance = xr.distance();
            let yr_distance = yr.distance();

            xr_distance
                .partial_cmp(&yr_distance)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }
}

#[derive(Debug, Clone)]
pub struct GeometryHitResult {
    pub ray_origin: Point3,
    pub ray_direction: Vector3,
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub front_face: bool,
    pub uv: Point2,
}

impl GeometryHitResult {
    pub fn new(
        ray: &Ray,
        distance: FloatType,
        surface_normal: Vector3,
        tangent: Vector3,
        bitangent: Vector3,
        front_face: bool,
        uv: Point2,
    ) -> Self {
        Self {
            ray_origin: ray.origin,
            ray_direction: ray.direction,
            hit_point: ray.origin + (distance * ray.direction),
            distance,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            uv,
        }
    }
}

impl IntersectResult for GeometryHitResult {
    fn ray_origin(&self) -> Point3 {
        self.ray_origin
    }

    fn distance(&self) -> FloatType {
        self.distance
    }

    fn hit_point(&self) -> Point3 {
        self.hit_point
    }

    fn surface_normal(&self) -> Vector3 {
        self.surface_normal
    }

    fn tangent(&self) -> Vector3 {
        self.tangent
    }

    fn bitangent(&self) -> Vector3 {
        self.bitangent
    }

    fn front_face(&self) -> bool {
        self.front_face
    }

    fn uv(&self) -> Point2 {
        self.uv
    }
}

impl Transformable for GeometryHitResult {
    type Target = Self;

    fn core_transform(mut self, transform: &Matrix4, _inverse_transform: &Matrix4) -> Self::Target {
        self.ray_origin = transform.transform_point(self.ray_origin);
        self.ray_direction = transform.transform_vector(self.ray_direction);
        self.hit_point = transform.transform_point(self.hit_point);
        self.distance = (self.hit_point - self.ray_origin).magnitude();
        self.surface_normal = transform.transform_vector(self.surface_normal).normalize();
        self.tangent = transform.transform_vector(self.tangent).normalize();
        self.bitangent = transform.transform_vector(self.bitangent).normalize();

        self
    }
}

impl DefaultSkinnable for GeometryHitResult {}

pub type SkinnedHitResult = <GeometryHitResult as Skinnable>::Target;
