use crate::color::Color;
use crate::hittable::HitResult;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::texture::Texture;
use crate::utils::*;

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

pub trait MaterialClone {
    fn box_clone(&self) -> Box<dyn Material>;
}

pub trait Material: Sync + Send + MaterialClone + std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitResult) -> Option<ScatterResult>;
}

impl<T: Material + Clone + 'static> MaterialClone for T {
    fn box_clone(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Material> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Debug)]
pub struct Lambertian(Box<dyn Texture>);

impl Lambertian {
    pub fn new(texture: Box<dyn Texture>) -> Self {
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
        let reflected = reflect(ray_in.direction.normalize(), hit_record.surface_normal)
            + self.fuzz() * random_in_unit_sphere();
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
        let unit_ray_direction = ray_in.direction.normalize();

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
