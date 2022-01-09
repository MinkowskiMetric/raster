use crate::{math::*, GeometryHitResult, Ray, SkinnedHitResult};

pub trait PrimitiveIntersection {
    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType)
        -> Option<GeometryHitResult>;
}

pub trait VisibleIntersection {
    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<SkinnedHitResult>;
}
