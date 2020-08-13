use crate::math::*;
use crate::Material;

#[derive(Debug)]
pub struct HitResult<'a> {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub u: FloatType,
    pub v: FloatType,
}
