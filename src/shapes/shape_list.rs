use super::{HitResult, Hittable, SharedHittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::TracingStats;

use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ShapeList {
    shapes: Box<[SharedHittable]>,
}

impl ShapeList {
    pub fn from_shapes(shapes: impl IntoIterator<Item = SharedHittable>) -> Self {
        let shapes = shapes.into_iter().collect::<Vec<_>>().into_boxed_slice();

        Self { shapes }
    }
}

impl Hittable for ShapeList {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max, stats))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.shapes
            .iter()
            .map(|a| a.bounding_box(t0, t1).clone())
            .my_fold_first(|a, b| BoundingBox::surrounding_box(&a, &b))
            .unwrap_or(BoundingBox::empty_box())
    }
}

pub mod factories {
    use super::*;

    pub fn shape_list(shapes: impl IntoIterator<Item = SharedHittable>) -> Arc<ShapeList> {
        Arc::new(ShapeList::from_shapes(shapes))
    }
}
