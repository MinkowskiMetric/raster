use crate::aabb::BoundingBox;
use crate::hittable::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;

#[derive(Clone, Debug)]
pub struct ShapeList {
    shapes: Box<[Box<dyn Hittable>]>,
    bounding_box: BoundingBox,
}

impl ShapeList {
    pub fn from_shapes(shapes: impl IntoIterator<Item = Box<dyn Hittable>>) -> Self {
        let shapes = shapes.into_iter().collect::<Vec<_>>().into_boxed_slice();
        let bounding_box = shapes.iter().map(|a| a.bounding_box().clone()).my_fold_first(|a,b| BoundingBox::surrounding_box(&a, &b)).unwrap_or(BoundingBox::empty_box());

        Self { shapes, bounding_box }
    }
}

impl Hittable for ShapeList {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}