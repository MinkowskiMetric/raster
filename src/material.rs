use crate::color::Color;
use crate::ray_scanner::Ray;
use crate::scene::HitResult;
use crate::utils::*;

use cgmath::prelude::*;

fn reflect(v: cgmath::Vector3<f32>, n: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    return v - (2.0 * v.dot(n) * n);
}

fn refract(
    v: cgmath::Vector3<f32>,
    n: cgmath::Vector3<f32>,
    etai_over_etat: f32,
) -> cgmath::Vector3<f32> {
    let cos_theta = (-v).dot(n);
    let r_out_perp = etai_over_etat * (v + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.magnitude2()).abs().sqrt() * n;

    r_out_perp + r_out_parallel
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

pub struct ScatterResult {
    pub attenuation: cgmath::Vector3<f32>,
    pub scattered: Ray,
}

pub trait Material {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult>;
}

pub struct Lambertian(Color);

impl Lambertian {
    pub fn new(color: Color) -> Self {
        Lambertian(color)
    }

    pub fn color(&self) -> &Color {
        &self.0
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult> {
        let target = hit_record.hit_point + hit_record.surface_normal + random_unit_vector();
        Some(ScatterResult {
            attenuation: cgmath::Vector4::from(*self.color()).truncate(),
            scattered: Ray::new(hit_record.hit_point, target - hit_record.hit_point),
        })
    }
}

pub struct Metal(Color, f32);

impl Metal {
    pub fn new(color: Color, fuzz: f32) -> Self {
        Metal(color, if fuzz < 1.0 { fuzz } else { 1.0 })
    }

    pub fn color(&self) -> &Color {
        &self.0
    }

    pub fn fuzz(&self) -> f32 {
        self.1
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult> {
        let reflected = reflect(ray_in.direction.normalize(), hit_record.surface_normal)
            + self.fuzz() * random_in_unit_sphere();
        if reflected.dot(hit_record.surface_normal) > 0.0 {
            Some(ScatterResult {
                attenuation: cgmath::Vector4::from(*self.color()).truncate(),
                scattered: Ray::new(hit_record.hit_point, reflected.normalize()),
            })
        } else {
            None
        }
    }
}

pub struct Dielectric(f32);

impl Dielectric {
    pub fn new(ri: f32) -> Self {
        Self(ri)
    }

    pub fn refractive_index(&self) -> f32 {
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
        let unit_ray_direction = ray_in.direction.normalize();

        let cos_theta = -unit_ray_direction.dot(hit_record.surface_normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        if etai_over_etat * sin_theta > 1.0
            || random_in_range(0.0, 1.0) < schlick(cos_theta, etai_over_etat)
        {
            let reflected = reflect(unit_ray_direction, hit_record.surface_normal);

            Some(ScatterResult {
                attenuation: cgmath::vec3(1.0, 1.0, 1.0),
                scattered: Ray::new(hit_record.hit_point, reflected),
            })
        } else {
            let refracted = refract(
                unit_ray_direction,
                hit_record.surface_normal,
                etai_over_etat,
            );

            Some(ScatterResult {
                attenuation: cgmath::vec3(1.0, 1.0, 1.0),
                scattered: Ray::new(hit_record.hit_point, refracted),
            })
        }
    }
}
