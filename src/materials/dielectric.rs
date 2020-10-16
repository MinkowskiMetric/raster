use super::utils::*;
use super::{Material, PartialScatterResult, ScatterResult};
use crate::math::*;
use crate::utils::*;
use crate::{GeometryHitResult, HitResult, Ray};

#[derive(Clone, Debug)]
pub struct Dielectric(FloatType);

impl Dielectric {
    pub fn new(ri: FloatType) -> Self {
        Self(ri)
    }

    pub fn refractive_index(&self) -> FloatType {
        self.0
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, hit_record: HitResult) -> Option<ScatterResult> {
        let etai_over_etat = if hit_record.front_face() {
            1.0 / self.refractive_index()
        } else {
            self.refractive_index()
        };
        let unit_ray_direction = ray_in.direction.into_vector().normalize();

        let cos_theta = -unit_ray_direction.dot(hit_record.surface_normal()).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        if etai_over_etat * sin_theta > 1.0
            || random_in_range(0.0, 1.0) < schlick(cos_theta, etai_over_etat)
        {
            let reflected = reflect(unit_ray_direction, hit_record.surface_normal());

            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: vec3(1.0, 1.0, 1.0),
                },
                scattered: Ray::new(hit_record.hit_point(), reflected, ray_in.time),
            })
        } else {
            let refracted = refract(
                unit_ray_direction,
                hit_record.surface_normal(),
                etai_over_etat,
            );

            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: vec3(1.0, 1.0, 1.0),
                },
                scattered: Ray::new(hit_record.hit_point(), refracted, ray_in.time),
            })
        }
    }
}

pub mod factories {
    use super::*;

    pub fn dielectric(ri: FloatType) -> Dielectric {
        Dielectric::new(ri)
    }
}
