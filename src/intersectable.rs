use crate::{math::*, IntersectResult, IntersectResultIteratorOps, Ray};

pub trait Intersectable {
    type Result: IntersectResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result>;
}

pub trait IntersectableIteratorOps {
    type Result: IntersectResult;

    fn intersect(self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result>;
}

impl<B: Intersectable, IntoIter: IntoIterator<Item = B>> IntersectableIteratorOps for IntoIter {
    type Result = B::Result;

    fn intersect(self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.into_iter()
            .filter_map(|shape| shape.intersect(ray, t_min, t_max))
            .nearest()
    }
}
