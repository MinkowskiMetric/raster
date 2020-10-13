use super::{factories::*, CompoundShape, GeometryModifier, GeometryWrapper, HitResult};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;

#[derive(Debug, Clone)]
pub struct Scale(Vector3);

impl Scale {
    fn scale(&self) -> &Vector3 {
        &self.0
    }

    fn unscale_point(&self, p: Point3) -> Point3 {
        let scale = self.scale();
        Point3::new(p.x / scale.x, p.y / scale.y, p.z / scale.z)
    }

    fn unscale_vector(&self, p: Vector3) -> Vector3 {
        let scale = self.scale();
        Vector3::new(p.x / scale.x, p.y / scale.y, p.z / scale.z)
    }

    fn scale_point(&self, p: Point3) -> Point3 {
        let scale = self.scale();
        Point3::new(p.x * scale.x, p.y * scale.y, p.z * scale.z)
    }

    fn scale_vector(&self, p: Vector3) -> Vector3 {
        let scale = self.scale();
        Vector3::new(p.x * scale.x, p.y * scale.y, p.z * scale.z)
    }
}

impl GeometryModifier for Scale {
    fn process_input_ray(&self, ray: &Ray) -> Ray {
        let scaled_origin = self.unscale_point(ray.origin.into_point());
        let scaled_direction = self.unscale_vector(ray.direction.into_vector());
        Ray::new(scaled_origin, scaled_direction, ray.time)
    }

    fn process_hit_result<'a>(
        &self,
        original_ray: &Ray,
        _modified_ray: &Ray,
        mut hit_result: HitResult<'a>,
    ) -> HitResult<'a> {
        hit_result.hit_point = self.scale_point(hit_result.hit_point);
        hit_result.surface_normal = self.scale_vector(hit_result.surface_normal).normalize();
        hit_result.distance = (hit_result.hit_point - original_ray.origin.into_point()).magnitude();

        hit_result
    }

    fn translate_bounding_box(&self, child_bounding_box: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            self.scale_point(child_bounding_box.min_point().into_point()),
            self.scale_point(child_bounding_box.max_point().into_point()),
        )
    }
}

pub mod factories {
    use super::*;

    pub fn scale<T: 'static + CompoundShape>(scale: Vector3, child: T) -> GeometryWrapper<Scale> {
        geometry_wrapper(Scale(scale), child)
    }
}
