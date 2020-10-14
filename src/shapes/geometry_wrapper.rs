use super::{HitResult, Shape, SimpleShape, UntransformedShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

pub trait GeometryModifier: Sync + Send + std::fmt::Debug + Clone {
    fn process_input_ray(&self, ray: &Ray) -> Ray;
    fn process_hit_result<'a>(
        &self,
        original_ray: &Ray,
        modified_ray: &Ray,
        hit_result: HitResult<'a>,
    ) -> HitResult<'a>;
    fn translate_bounding_box(&self, bounding_box: BoundingBox) -> BoundingBox;
}

#[derive(Debug)]
pub struct GeometryWrapper<Modifier: GeometryModifier> {
    modifier: Modifier,
    child: Box<dyn Shape>,
}

impl<Modifier: 'static + GeometryModifier> Shape for GeometryWrapper<Modifier> {
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

impl<Modifier: 'static + GeometryModifier> SimpleShape for GeometryWrapper<Modifier> {}
impl<Modifier: 'static + GeometryModifier> UntransformedShape for GeometryWrapper<Modifier> {}

pub mod factories {
    use super::*;

    pub fn geometry_wrapper<Modifier: GeometryModifier, Child: 'static + Shape>(
        modifier: Modifier,
        child: Child,
    ) -> GeometryWrapper<Modifier> {
        GeometryWrapper {
            modifier,
            child: Box::new(child),
        }
    }
}
