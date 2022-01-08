use crate::Ray;
use crate::{
    math::*, sky::Sky, BoundingBox, Intersectable, KDTree, SkinnedHitResult, TimeDependentBounded,
};
use crate::{Camera, PreparedCamera};
use crate::{CompoundVisible, DynVisible, Visible};

pub struct Scene {
    camera: Camera,
    sky: Sky,
    shapes: CompoundVisible,
}

impl Scene {
    pub fn new(camera: Camera, sky: Sky, shapes: CompoundVisible) -> Self {
        Scene {
            camera,
            sky,
            shapes: shapes.decompose(),
        }
    }
}

pub struct PreparedScene {
    camera: PreparedCamera,
    sky: Sky,
    root_volume: KDTree<DynVisible>,
}

impl PreparedScene {
    pub fn make(scene: Scene, t0: FloatType, t1: FloatType) -> Self {
        Self {
            camera: PreparedCamera::make(scene.camera, t0, t1),
            sky: scene.sky,
            root_volume: KDTree::snapshot(scene.shapes, t0, t1),
        }
    }

    pub fn camera(&self) -> &PreparedCamera {
        &self.camera
    }

    pub fn sky(&self) -> &Sky {
        &self.sky
    }
}

impl Intersectable for PreparedScene {
    type Result = SkinnedHitResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<SkinnedHitResult> {
        self.root_volume.intersect(ray, t_min, t_max)
    }
}

impl TimeDependentBounded for PreparedScene {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.root_volume.time_dependent_bounding_box(t0, t1)
    }
}
