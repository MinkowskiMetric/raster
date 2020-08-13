use super::utils::*;
use super::{Material, PartialScatterResult, ScatterResult};
use crate::math::*;
use crate::utils::*;
use crate::{Color, HitResult, Ray};

#[derive(Clone, Debug)]
pub struct Metal(Color, FloatType);

impl Metal {
    pub fn new(color: Color, fuzz: FloatType) -> Self {
        Metal(color, if fuzz < 1.0 { fuzz } else { 1.0 })
    }

    pub fn color(&self) -> &Color {
        &self.0
    }

    pub fn fuzz(&self) -> FloatType {
        self.1
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult> {
        let reflected = reflect(
            ray_in.direction.into_vector().normalize(),
            hit_record.surface_normal,
        ) + self.fuzz() * random_in_unit_sphere();
        if reflected.dot(hit_record.surface_normal) > 0.0 {
            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: cgmath::Vector4::from(*self.color()).truncate(),
                },
                scattered: Ray::new(hit_record.hit_point, reflected.normalize(), ray_in.time),
            })
        } else {
            None
        }
    }
}

pub mod factories {
    use super::*;

    pub fn metal(color: Color, fuzz: FloatType) -> Metal {
        Metal::new(color, fuzz)
    }
}
