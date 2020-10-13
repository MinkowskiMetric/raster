use super::{factories::*, CompoundShape, GeometryModifier, GeometryWrapper, HitResult};
use crate::ray_scanner::Ray;
use crate::BoundingBox;

#[derive(Debug, Clone)]
pub struct InvertNormal();

impl GeometryModifier for InvertNormal {
    fn process_input_ray(&self, ray: &Ray) -> Ray {
        ray.clone()
    }

    fn process_hit_result<'a>(
        &self,
        _original_ray: &Ray,
        _modified_ray: &Ray,
        mut hit_result: HitResult<'a>,
    ) -> HitResult<'a> {
        hit_result.front_face = !hit_result.front_face;

        hit_result
    }

    fn translate_bounding_box(&self, child_bounding_box: BoundingBox) -> BoundingBox {
        child_bounding_box
    }
}

pub mod factories {
    use super::*;

    pub fn invert_normal<T: 'static + CompoundShape>(child: T) -> GeometryWrapper<InvertNormal> {
        geometry_wrapper(InvertNormal(), child)
    }
}
