use super::{factories::*, HitResult, Hittable, ShapeList, SharedHittable};
use crate::math::*;
use crate::BoundingBox;
use crate::TracingStats;
use crate::{Ray, SharedMaterial};

use std::sync::Arc;

#[derive(Debug)]
pub struct BoxShape {
    pt_min: Point3,
    pt_max: Point3,
    shapes: ShapeList,
}

impl BoxShape {
    pub fn new(pt_min: Point3, pt_max: Point3, material: SharedMaterial) -> Self {
        let sides: Vec<SharedHittable> = vec![
            xy_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.y, pt_max.y),
                pt_max.z,
                material.clone(),
            ),
            xy_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.y, pt_max.y),
                pt_min.z,
                material.clone(),
            ),
            xz_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.z, pt_max.z),
                pt_max.y,
                material.clone(),
            ),
            xz_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.z, pt_max.z),
                pt_min.y,
                material.clone(),
            ),
            yz_rectangle(
                (pt_min.y, pt_max.y),
                (pt_min.z, pt_max.z),
                pt_max.x,
                material.clone(),
            ),
            yz_rectangle(
                (pt_min.y, pt_max.y),
                (pt_min.z, pt_max.z),
                pt_min.x,
                material.clone(),
            ),
        ];

        Self {
            pt_min,
            pt_max,
            shapes: ShapeList::from_shapes(sides),
        }
    }
}

impl Hittable for BoxShape {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        self.shapes.intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        BoundingBox::new(self.pt_min, self.pt_max)
    }
}

pub mod factories {
    use super::*;

    pub fn box_shape(pt_min: Point3, pt_max: Point3, material: SharedMaterial) -> Arc<BoxShape> {
        Arc::new(BoxShape::new(pt_min, pt_max, material))
    }
}
