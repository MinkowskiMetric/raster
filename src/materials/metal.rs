use super::utils::*;
use super::{Material, PartialScatterResult, ScatterResult};
use crate::math::*;
use crate::utils::*;
use crate::{Color, PrimitiveHitResult, Ray, Texture};

#[derive(Debug)]
pub struct Metal<T: Texture>(T, FloatType);

impl<T: Texture> Metal<T> {
    pub fn new(texture: T, fuzz: FloatType) -> Self {
        Metal(texture, if fuzz < 1.0 { fuzz } else { 1.0 })
    }

    pub fn texture(&self) -> &T {
        &self.0
    }

    pub fn fuzz(&self) -> FloatType {
        self.1
    }
}

impl<T: Texture + Clone> Clone for Metal<T> {
    fn clone(&self) -> Self {
        Metal(self.0.clone(), self.1)
    }
}

impl<T: Texture> Material for Metal<T> {
    fn scatter(&self, ray_in: &Ray, hit_record: PrimitiveHitResult) -> Option<ScatterResult> {
        let reflected = reflect(ray_in.direction.normalize(), hit_record.surface_normal())
            + self.fuzz() * random_in_unit_sphere();
        let color = self
            .texture()
            .value(hit_record.hit_point(), *hit_record.uv());
        if reflected.dot(hit_record.surface_normal()) > 0.0 {
            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: cgmath::Vector4::from(color).truncate(),
                },
                scattered: Ray::new(hit_record.hit_point(), reflected.normalize(), ray_in.time),
            })
        } else {
            None
        }
    }
}

pub mod factories {
    use super::*;
    use crate::factories::*;
    use crate::textures::SolidTexture;

    pub fn metal_with_texture<T: Texture>(texture: T, fuzz: FloatType) -> Metal<T> {
        Metal::new(texture, fuzz)
    }

    pub fn metal(color: Color, fuzz: FloatType) -> Metal<SolidTexture> {
        metal_with_texture(solid_texture(color), fuzz)
    }
}
