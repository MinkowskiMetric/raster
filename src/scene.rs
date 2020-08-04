use crate::aabb::BoundingBox;
use crate::camera::{Camera, PreparedCamera};
use crate::hittable::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::shape_list::ShapeList;
use crate::stats::TracingStats;
use crate::volume::Volume;

#[derive(Clone, Debug)]
enum RootShape {
    Volume(Volume),
    ShapeList(ShapeList),
}

impl RootShape {
    pub fn from_shapes(
        enable_spatial_partitioning: bool,
        t0: FloatType,
        t1: FloatType,
        shapes: impl IntoIterator<Item = Box<dyn Hittable>>,
    ) -> Self {
        if enable_spatial_partitioning {
            RootShape::Volume(Volume::from_shapes(shapes, t0, t1))
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
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        match self {
            RootShape::Volume(v) => v.intersect(ray, t_min, t_max, stats),
            RootShape::ShapeList(s) => s.intersect(ray, t_min, t_max, stats),
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        match self {
            RootShape::Volume(v) => v.bounding_box(t0, t1),
            RootShape::ShapeList(s) => s.bounding_box(t0, t1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    camera: Camera,
    enable_spatial_partitioning: bool,
    shapes: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new(
        camera: Camera,
        enable_spatial_partitioning: bool,
        shapes: impl IntoIterator<Item = Box<dyn Hittable>>,
    ) -> Self {
        Scene {
            camera,
            enable_spatial_partitioning,
            shapes: shapes.into_iter().collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreparedScene {
    camera: PreparedCamera,
    root_volume: RootShape,
}

impl PreparedScene {
    pub fn make(scene: Scene, t0: FloatType, t1: FloatType) -> Self {
        let root_volume =
            RootShape::from_shapes(scene.enable_spatial_partitioning, t0, t1, scene.shapes);

        Self {
            camera: PreparedCamera::make(scene.camera, t0, t1),
            root_volume,
        }
    }

    pub fn camera(&self) -> &PreparedCamera {
        &self.camera
    }
}

impl Hittable for PreparedScene {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        self.root_volume.intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.root_volume.bounding_box(t0, t1)
    }
}
