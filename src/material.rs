use crate::color::Color;
use crate::hittable::HitResult;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::texture::{SharedTexture, Texture};
use crate::utils::*;

use std::sync::Arc;

fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    return v - (2.0 * v.dot(n) * n);
}

fn refract(v: Vector3, n: Vector3, etai_over_etat: FloatType) -> Vector3 {
    let cos_theta = (-v).dot(n);
    let r_out_perp = etai_over_etat * (v + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.magnitude2()).abs().sqrt() * n;

    r_out_perp + r_out_parallel
}

fn schlick(cosine: FloatType, ref_idx: FloatType) -> FloatType {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PartialScatterResult {
    pub attenuation: Vector3,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScatterResult {
    pub partial: PartialScatterResult,
    pub scattered: Ray,
}

pub trait Material: Sync + Send + std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult>;

    fn emitted(&self, _p: Point3, _u: FloatType, _v: FloatType) -> Color {
        Color::BLACK
    }
}

pub type SharedMaterial = Arc<dyn Material>;

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
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult> {
        let etai_over_etat = if hit_record.front_face {
            1.0 / self.refractive_index()
        } else {
            self.refractive_index()
        };
        let unit_ray_direction = ray_in.direction.into_vector().normalize();

        let cos_theta = -unit_ray_direction.dot(hit_record.surface_normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        if etai_over_etat * sin_theta > 1.0
            || random_in_range(0.0, 1.0) < schlick(cos_theta, etai_over_etat)
        {
            let reflected = reflect(unit_ray_direction, hit_record.surface_normal);

            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: vec3(1.0, 1.0, 1.0),
                },
                scattered: Ray::new(hit_record.hit_point, reflected, ray_in.time),
            })
        } else {
            let refracted = refract(
                unit_ray_direction,
                hit_record.surface_normal,
                etai_over_etat,
            );

            Some(ScatterResult {
                partial: PartialScatterResult {
                    attenuation: vec3(1.0, 1.0, 1.0),
                },
                scattered: Ray::new(hit_record.hit_point, refracted, ray_in.time),
            })
        }
    }
}

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

pub mod materials {
    use super::*;

    pub fn lambertian(texture: SharedTexture) -> Arc<Lambertian> {
        Arc::new(Lambertian::new(texture))
    }

    pub fn metal(color: Color, fuzz: FloatType) -> Arc<Metal> {
        Arc::new(Metal::new(color, fuzz))
    }

    pub fn dielectric(ri: FloatType) -> Arc<Dielectric> {
        Arc::new(Dielectric::new(ri))
    }

    pub fn diffuse_light(texture: SharedTexture) -> Arc<DiffuseLight> {
        Arc::new(DiffuseLight::new(texture))
    }
}
