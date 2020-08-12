use super::{Material, ScatterResult};
use crate::math::*;
use crate::{Color, HitResult, Ray, SharedTexture, Texture};

use std::sync::Arc;

#[derive(Debug)]
pub struct DiffuseLight(SharedTexture);

impl DiffuseLight {
    pub fn new(emit: SharedTexture) -> Self {
        Self(emit)
    }

    pub fn emit(&self) -> &dyn Texture {
        self.0.as_ref()
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, p: Point3, u: FloatType, v: FloatType) -> Color {
        self.emit().value(p, u, v)
    }

    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitResult) -> Option<ScatterResult> {
        None
    }
}

pub mod factories {
    use super::*;

    pub fn diffuse_light(texture: SharedTexture) -> Arc<DiffuseLight> {
        Arc::new(DiffuseLight::new(texture))
    }
}
