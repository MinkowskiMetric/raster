use super::{factories::*, shapes, GeometryObject, ShapeList};
use crate::math::*;
use crate::Material;

#[derive(Debug, Clone)]
pub struct BoxShape {
    pt_min: Point3,
    pt_max: Point3,
    shapes: ShapeList,
}

impl BoxShape {
    pub fn new<T: 'static + Material + Clone>(pt_min: Point3, pt_max: Point3, material: T) -> Self {
        let shapes = shapes![
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
            shapes,
        }
    }
}

impl GeometryObject for BoxShape {
    type GeometryIterator = <ShapeList as GeometryObject>::GeometryIterator;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        self.shapes.into_geometry_iterator()
    }
}

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
