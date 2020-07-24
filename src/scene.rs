use crate::aabb::BoundingBox;
use crate::camera::Camera;
use crate::hittable::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::volume::Volume;
use crate::shape_list::ShapeList;

#[derive(Clone, Debug)]
enum RootShape {
    Volume(Volume),
    ShapeList(ShapeList),
}

impl RootShape {
    pub fn from_shapes(enable_spatial_partitioning: bool, shapes: impl IntoIterator<Item = Box<dyn Hittable>>) -> Self {
        if enable_spatial_partitioning {
            RootShape::Volume(Volume::from_shapes(shapes))
        } else {
            RootShape::ShapeList(ShapeList::from_shapes(shapes))
        }
    }
}

impl Hittable for RootShape {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>> {
        match self {
            RootShape::Volume(v) => v.intersect(ray, t_min, t_max),
            RootShape::ShapeList(s) => s.intersect(ray, t_min, t_max),
        }
    }

    fn bounding_box(&self) -> &BoundingBox {
        match self {
            RootShape::Volume(v) => v.bounding_box(),
            RootShape::ShapeList(s) => s.bounding_box(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    camera: Camera,
    root_volume: RootShape,
}

impl Scene {
    pub fn new(camera: Camera, enable_spatial_partitioning: bool, shapes: impl IntoIterator<Item = Box<dyn Hittable>>) -> Self {
        let root_volume = RootShape::from_shapes(enable_spatial_partitioning, shapes);

        Scene {
            camera,
            root_volume,
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

impl Hittable for Scene {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>> {
        self.root_volume.intersect(ray, t_min, t_max)
    }

    fn bounding_box(&self) -> &BoundingBox {
        self.root_volume.bounding_box()
    }
}
