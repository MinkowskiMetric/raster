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

pub trait HittableClone {
    fn box_clone(&self) -> Box<dyn Hittable>;
}

pub trait Hittable: Sync + Send + HittableClone {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<HitResult<'a>>;
}

impl<T: Hittable + Clone + 'static> HittableClone for T {
    fn box_clone(&self) -> Box<dyn Hittable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Hittable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
