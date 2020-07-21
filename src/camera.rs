use crate::ray_scanner::Ray;
use cgmath::prelude::*;

pub struct Camera {
    /*position: cgmath::Vector3<f32>,
    inv_width: f32,
    inv_height: f32,
    aspect_ratio: f32,
    angle: f32,*/
    origin: cgmath::Vector3<f32>,
    lower_left_corner: cgmath::Vector3<f32>,
    horizontal: cgmath::Vector3<f32>,
    vertical: cgmath::Vector3<f32>,
}

impl Camera {
    pub fn new(
        /*position: cgmath::Vector3<f32>,
        width: usize,
        height: usize,
        fov: cgmath::Rad<f32>,*/
    ) -> Self {
        let aspect_ratio = 16.0 / 9.0;
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let origin = cgmath::vec3(0.0, 0.0, 0.0);
        let horizontal = cgmath::vec3(viewport_width, 0.0, 0.0);
        let vertical = cgmath::vec3(0.0, viewport_height, 0.0);
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - cgmath::vec3(0.0, 0.0, focal_length);

        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        }

        /*let inv_width = 1.0_f32 / (width as f32);
        let inv_height = 1.0_f32 / (height as f32);
        let aspect_ratio = inv_height / inv_width;

        let angle = (fov / 2.0).tan();

        Self {
            position,
            inv_width,
            inv_height,
            aspect_ratio,
            angle,
        }*/
    }

    pub fn make_ray(&self, u: f32, v: f32) -> Ray {
        let direction = self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin;
        Ray::new(self.origin, direction.normalize())
        /*let vx =
            (2_f32 * ((x + 0.5_f32) * self.inv_width) - 1_f32) * self.angle * self.aspect_ratio;
        let vy = (1_f32 - 2_f32 * ((y + 0.5_f32) * self.inv_height)) * self.angle;

        let ray_direction = cgmath::vec3(vx, vy, 1.0_f32);
        ray_direction.normalize()*/
    }
}
