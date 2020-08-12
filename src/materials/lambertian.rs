use super::{Material, PartialScatterResult, ScatterResult};
use crate::utils::*;
use crate::{HitResult, Ray, SharedTexture, Texture};

use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Lambertian(SharedTexture);

impl Lambertian {
    pub fn new(texture: SharedTexture) -> Self {
        Lambertian(texture)
    }

    pub fn albedo(&self) -> &dyn Texture {
        self.0.as_ref()
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult> {
        let target = hit_record.hit_point + hit_record.surface_normal + random_unit_vector();
        let color = self
            .albedo()
            .value(hit_record.hit_point, hit_record.u, hit_record.v);
        Some(ScatterResult {
            partial: PartialScatterResult {
                attenuation: cgmath::Vector4::from(color).truncate(),
            },
            scattered: Ray::new(
                hit_record.hit_point,
                target - hit_record.hit_point,
                ray_in.time,
            ),
        })
    }
}

pub mod factories {
    use super::*;

    pub fn lambertian(texture: SharedTexture) -> Arc<Lambertian> {
        Arc::new(Lambertian::new(texture))
    }
}
