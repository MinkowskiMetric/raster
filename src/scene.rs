use crate::camera::Camera;
use crate::hittable::Hittable;
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

    pub fn get_shapes<'a>(&'a self, _ray: &'a Ray) -> impl Iterator<Item = &'a Box<dyn Hittable>> {
        self.shapes().iter()
    }
}
