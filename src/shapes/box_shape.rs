use super::{
    factories::*, shapes, CompoundShape, HitResult, Primitive, Shape, ShapeList, UntransformedShape,
};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::RenderStatsCollector;
use crate::{BoundingBox, Material};

#[derive(Debug)]
pub struct BoxShape {
    pt_min: Point3,
    pt_max: Point3,
    shapes: ShapeList,
}

impl BoxShape {
    pub fn new<T: 'static + Material + Clone>(pt_min: Point3, pt_max: Point3, material: T) -> Self {
        let shapes = shapes![
            xy_rectangle((pt_min.x, pt_max.x), (pt_min.y, pt_max.y), pt_max.z)
                .apply_material(material.clone(),),
            xy_rectangle((pt_min.x, pt_max.x), (pt_min.y, pt_max.y), pt_min.z)
                .apply_material(material.clone(),),
            xz_rectangle((pt_min.x, pt_max.x), (pt_min.z, pt_max.z), pt_max.y)
                .apply_material(material.clone(),),
            xz_rectangle((pt_min.x, pt_max.x), (pt_min.z, pt_max.z), pt_min.y)
                .apply_material(material.clone(),),
            yz_rectangle((pt_min.y, pt_max.y), (pt_min.z, pt_max.z), pt_max.x)
                .apply_material(material.clone(),),
            yz_rectangle((pt_min.y, pt_max.y), (pt_min.z, pt_max.z), pt_min.x)
                .apply_material(material.clone(),),
        ];

        Self {
            pt_min,
            pt_max,
            shapes,
        }
    }
}

impl Shape for BoxShape {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.shapes.intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.shapes.bounding_box(t0, t1)
    }
}

impl CompoundShape for BoxShape {
    type GeometryIterator = <ShapeList as CompoundShape>::GeometryIterator;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        self.shapes.into_geometry_iterator()
    }
}

impl UntransformedShape for BoxShape {}

pub mod factories {
    use super::*;

    pub fn box_shape<T: 'static + Material + Clone>(
        pt_min: Point3,
        pt_max: Point3,
        material: T,
    ) -> BoxShape {
        BoxShape::new(pt_min, pt_max, material)
    }
}
