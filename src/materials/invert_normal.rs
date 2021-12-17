use super::{Material, ScatterResult};
use crate::{IntersectResult, Ray};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct InvertNormal<M: Material + Clone>(M);

impl<M: Material + Clone> InvertNormal<M> {
    pub fn new(material: M) -> Self {
        Self(material)
    }
}

impl<M: Material + Clone> Material for InvertNormal<M> {
    fn scatter(&self, ray_in: &Ray, hit_record: &dyn IntersectResult) -> Option<ScatterResult> {
        let mut hit_record = hit_record.as_geometry_hit_result();
        hit_record.set_front_face(!hit_record.front_face());
        self.0.scatter(ray_in, &hit_record)
    }
}

pub mod factories {
    use super::*;

    pub fn invert_normal<M: Material + Clone>(material: M) -> InvertNormal<M> {
        InvertNormal::new(material)
    }
}
