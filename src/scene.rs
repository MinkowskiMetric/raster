use crate::camera::Camera;
use crate::material::Material;
use crate::ray_scanner::Ray;

pub struct HitResult<'a> {
    pub distance: f32,
    pub hit_point: cgmath::Vector3<f32>,
    pub surface_normal: cgmath::Vector3<f32>,
    pub front_face: bool,
    pub material: &'a Box<dyn Material>,
}

pub trait Shape {
    fn intersect<'a>(&'a self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult<'a>>;
}

pub struct Scene {
    camera: Camera,
    shapes: Box<[Box<dyn Shape>]>,
}

impl Scene {
    pub fn new(camera: Camera, shapes: Vec<Box<dyn Shape>>) -> Self {
        let shapes = shapes.into_boxed_slice();

        Scene { camera, shapes }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn shapes(&self) -> &[Box<dyn Shape>] {
        &self.shapes
    }

    pub fn get_shapes<'a>(&'a self, _ray: &'a Ray) -> impl Iterator<Item = &'a Box<dyn Shape>> {
        self.shapes().iter()
    }
}
