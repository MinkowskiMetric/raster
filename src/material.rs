use crate::color::Color;
use crate::ray_scanner::Ray;
use crate::scene::HitResult;

use cgmath::prelude::*;
use rand::prelude::*;

fn random_in_unit_sphere() -> cgmath::Vector3<f32> {
    loop {
        let p = cgmath::vec3(random::<f32>(), random::<f32>(), random::<f32>());
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

fn reflect(v: cgmath::Vector3<f32>, n: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    return v - (2.0 * v.dot(n) * n);
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
        let target = hit_record.hit_point + hit_record.surface_normal + random_in_unit_sphere();
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
        let reflected = reflect(ray_in.direction.normalize(), hit_record.surface_normal) + self.fuzz() * random_in_unit_sphere();
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
