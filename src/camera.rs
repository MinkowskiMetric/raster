use crate::ray_scanner::Ray;
use crate::utils::*;
use cgmath::prelude::*;

pub struct Camera {
    origin: cgmath::Point3<f32>,
    lower_left_corner: cgmath::Point3<f32>,
    horizontal: cgmath::Vector3<f32>,
    vertical: cgmath::Vector3<f32>,
    u: cgmath::Vector3<f32>,
    v: cgmath::Vector3<f32>,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        lookfrom: cgmath::Point3<f32>,
        lookat: cgmath::Point3<f32>,
        vup: cgmath::Vector3<f32>,
        fov: cgmath::Rad<f32>,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let h = (fov / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn make_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        let direction =
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset;
        Ray::new(self.origin + offset, direction.normalize())
    }
}
