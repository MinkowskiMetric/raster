use crate::camera::Camera;
use crate::hittable::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone)]
pub struct Scene {
    camera: Camera,
    shapes: Box<[Box<dyn Hittable>]>,
}

impl Scene {
    pub fn new(camera: Camera, shapes: Vec<Box<dyn Hittable>>) -> Self {
        let shapes = shapes.into_boxed_slice();

        Scene { camera, shapes }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn shapes(&self) -> &[Box<dyn Hittable>] {
        &self.shapes
    }
}

impl Hittable for Scene {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>> {
        self.shapes()
            .iter()
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
