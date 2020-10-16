use super::{Material, ScatterResult};
use crate::math::*;
use crate::{Color, PrimitiveHitResult, Ray, Texture};

#[derive(Debug, Clone)]
pub struct DiffuseLight<T: 'static + Texture + Clone>(T);

impl<T: 'static + Texture + Clone> DiffuseLight<T> {
    pub fn new(emit: T) -> Self {
        Self(emit)
    }

    pub fn emit(&self) -> &T {
        &self.0
    }
}

impl<T: 'static + Texture + Clone> Material for DiffuseLight<T> {
    fn emitted(&self, p: Point3, uv: (FloatType, FloatType)) -> Color {
        self.emit().value(p, uv)
    }

    fn scatter(&self, _ray_in: &Ray, _hit_record: PrimitiveHitResult) -> Option<ScatterResult> {
        None
    }
}

pub mod factories {
    use super::*;

    pub fn diffuse_light<T: 'static + Texture + Clone>(texture: T) -> DiffuseLight<T> {
        DiffuseLight::new(texture)
    }
}
