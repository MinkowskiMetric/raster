use crate::camera::Camera;
use crate::material::Material;
use crate::math::*;
use crate::ray_scanner::Ray;

pub struct HitResult<'a> {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub front_face: bool,
    pub material: &'a Box<dyn Material>,
}

pub trait ShapeClone {
    fn box_clone(&self) -> Box<dyn Shape>;
}

pub trait Shape: Sync + Send + ShapeClone {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>>;
}

impl<T: Shape + Clone + 'static> ShapeClone for T {
    fn box_clone(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone)]
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
