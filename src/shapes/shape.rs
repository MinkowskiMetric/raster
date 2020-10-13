use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

use super::HitResult;

pub trait Shape: Send + Sync + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait SimpleShape: Shape {}

impl<S: Shape> Shape for Box<S> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.as_ref().intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.as_ref().bounding_box(t0, t1)
    }
}

impl<S: SimpleShape> SimpleShape for Box<S> {}

pub trait CompoundShape: Shape {
    type GeometryIterator: Iterator<Item = Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator;
}

impl<S: SimpleShape + 'static> CompoundShape for S {
    type GeometryIterator = std::iter::Once<Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        let b: Box<dyn Shape> = Box::new(self);
        std::iter::once(b)
    }
}
