use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

use super::HitResult;

pub trait GeometryObject: std::fmt::Debug {
    type GeometryIterator: Iterator<Item = Box<dyn Hittable>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator;
}

pub trait HittableClone {
    fn box_clone(&self) -> Box<dyn Hittable>;
}

pub trait CoreHittable: HittableClone + Sync + Send + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait Hittable: HittableClone + Sync + Send + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

impl<T: 'static + Hittable + Clone> HittableClone for T {
    fn box_clone(&self) -> Box<dyn Hittable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Hittable> {
    fn clone(&self) -> Box<dyn Hittable> {
        self.as_ref().box_clone()
    }
}

impl<T: CoreHittable> Hittable for T {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.bounding_box(t0, t1)
    }
}

impl Hittable for Box<dyn Hittable> {
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

impl GeometryObject for Box<dyn Hittable> {
    type GeometryIterator = std::iter::Once<Box<dyn Hittable>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        std::iter::once(self)
    }
}

impl<T: 'static + CoreHittable> GeometryObject for T {
    type GeometryIterator = std::iter::Once<Box<dyn Hittable>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        let b: Box<dyn Hittable> = Box::new(self);
        std::iter::once(b)
    }
}
