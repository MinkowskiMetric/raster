use super::{Material, PartialScatterResult, ScatterResult};
use crate::utils::*;
use crate::{IntersectResult, Ray, Texture};

#[derive(Clone, Debug)]
pub struct Lambertian<T: 'static + Texture + Clone>(T);

impl<T: 'static + Texture + Clone> Lambertian<T> {
    pub fn new(texture: T) -> Self {
        Lambertian(texture)
    }

    pub fn albedo(&self) -> &T {
        &self.0
    }
}

impl<T: 'static + Texture + Clone> Material for Lambertian<T> {
    fn scatter(&self, ray_in: &Ray, hit_record: &dyn IntersectResult) -> Option<ScatterResult> {
        let target = hit_record.hit_point() + hit_record.surface_normal() + random_unit_vector();
        let color = self.albedo().value(hit_record.hit_point(), hit_record.uv());
        Some(ScatterResult {
            partial: PartialScatterResult {
                attenuation: cgmath::Vector4::from(color).truncate(),
            },
            scattered: Ray::new(
                hit_record.hit_point(),
                target - hit_record.hit_point(),
                ray_in.time,
            ),
        })
    }
}

pub mod factories {
    use super::*;

    pub fn lambertian<T: 'static + Texture + Clone>(texture: T) -> Lambertian<T> {
        Lambertian::new(texture)
    }
}
