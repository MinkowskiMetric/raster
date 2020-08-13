use super::{factories::*, GeometryModifier, GeometryObject, GeometryWrapper, HitResult};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;

#[derive(Debug, Clone)]
pub struct Translator(Vector3);

impl Translator {
    pub fn offset(&self) -> &Vector3 {
        &self.0
    }
}

impl GeometryModifier for Translator {
    fn process_input_ray(&self, ray: &Ray) -> Ray {
        Ray::new(
            ray.origin.into_point() - self.offset(),
            ray.direction.into_vector(),
            ray.time,
        )
    }

    fn process_hit_result<'a>(
        &self,
        _original_ray: &Ray,
        _modified_ray: &Ray,
        mut hit_result: HitResult<'a>,
    ) -> HitResult<'a> {
        hit_result.hit_point = hit_result.hit_point + self.offset();

        hit_result
    }

    fn translate_bounding_box(&self, child_bounding_box: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            (child_bounding_box.min_point().into_point() + self.offset()).into(),
            (child_bounding_box.max_point().into_point() + self.offset()).into(),
        )
    }
}

pub mod factories {
    use super::*;

    pub fn translate<T: 'static + GeometryObject + Clone>(
        offset: Vector3,
        child: T,
    ) -> GeometryWrapper<Translator, T> {
        geometry_wrapper(Translator(offset), child)
    }
}
