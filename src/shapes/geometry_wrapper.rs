use super::{GeometryObject, HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

pub trait GeometryModifier: Sync + Send + std::fmt::Debug {
    fn process_input_ray(&self, ray: &Ray) -> Ray;
    fn process_hit_result<'a>(
        &self,
        original_ray: &Ray,
        modified_ray: &Ray,
        hit_result: HitResult<'a>,
    ) -> HitResult<'a>;
    fn translate_bounding_box(&self, bounding_box: BoundingBox) -> BoundingBox;
}

#[derive(Debug, Clone)]
pub struct GeometryWrapper<Modifier: GeometryModifier + Clone, Child: GeometryObject + Clone> {
    modifier: Modifier,
    child: Child,
}

pub struct ModifiedGeometryIterator<
    Modifier: GeometryModifier + Clone,
    Iter: Iterator<Item = Box<dyn Hittable>>,
> {
    modifier: Modifier,
    iterator: Iter,
}

impl<Modifier: 'static + GeometryModifier + Clone, Iter: Iterator<Item = Box<dyn Hittable>>>
    ModifiedGeometryIterator<Modifier, Iter>
{
    fn map_child(&self, child: Box<dyn Hittable>) -> Box<dyn Hittable> {
        Box::new(GeometryWrapper {
            modifier: self.modifier.clone(),
            child,
        })
    }
}
impl<Modifier: 'static + GeometryModifier + Clone, Iter: Iterator<Item = Box<dyn Hittable>>>
    Iterator for ModifiedGeometryIterator<Modifier, Iter>
{
    type Item = Box<dyn Hittable>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|c| self.map_child(c))
    }
}

impl<Modifier: 'static + GeometryModifier + Clone, Child: 'static + GeometryObject + Clone>
    GeometryObject for GeometryWrapper<Modifier, Child>
{
    type GeometryIterator = ModifiedGeometryIterator<Modifier, Child::GeometryIterator>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        ModifiedGeometryIterator {
            modifier: self.modifier,
            iterator: self.child.into_geometry_iterator(),
        }
    }
}
impl<
        Modifier: 'static + GeometryModifier + Clone,
        Child: 'static + GeometryObject + Hittable + Clone,
    > Hittable for GeometryWrapper<Modifier, Child>
{
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        let input_ray = self.modifier.process_input_ray(ray);

        if let Some(hit_result) = self.child.intersect(&input_ray, t_min, t_max, stats) {
            Some(
                self.modifier
                    .process_hit_result(ray, &input_ray, hit_result),
            )
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.modifier
            .translate_bounding_box(self.child.bounding_box(t0, t1))
    }
}

pub mod factories {
    use super::*;

    pub fn geometry_wrapper<Modifier: GeometryModifier + Clone, Child: GeometryObject + Clone>(
        modifier: Modifier,
        child: Child,
    ) -> GeometryWrapper<Modifier, Child> {
        GeometryWrapper { modifier, child }
    }
}
