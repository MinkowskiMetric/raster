use crate::color::Color;
use crate::scene::{Scene, Shape};

use image::{Pixel, SurfaceMut};

use std::convert::TryInto;

pub struct Ray {
    pub origin: cgmath::Vector3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub inv_direction: cgmath::Vector3<f32>,
    pub sign: [usize; 3],
}

impl Ray {
    pub fn new(origin: cgmath::Vector3<f32>, direction: cgmath::Vector3<f32>) -> Self {
        let inv_direction = 1.0 / direction;
        let sign = [
            if inv_direction.x < 0.0 { 1 } else { 0 },
            if inv_direction.y < 0.0 { 1 } else { 0 },
            if inv_direction.z < 0.0 { 1 } else { 0 },
        ];

        Self {
            origin,
            direction,
            inv_direction,
            sign,
        }
    }
}

pub fn scan<P: Pixel + From<Color>>(image: &mut impl SurfaceMut<P>, scene: &Scene) {
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let direction = scene.camera().pixel_to_viewport(x, y);
        let ray = Ray::new(scene.camera().position(), direction);

        *pixel = trace(&ray, scene)
            .unwrap_or_else(|| {
                let unit_direction = ray.direction;
                let t = 0.5 * (unit_direction.y + 1.0);
                (((1.0 - t) * cgmath::vec3(1.0, 1.0, 1.0)) + (t * cgmath::vec3(0.5, 0.7, 1.0))).try_into().unwrap()
            })
            .gamma(2.0)
            .into();
    }
}

pub fn trace(ray: &Ray, scene: &Scene) -> Option<Color> {
    scene
        .intersect_shapes(&ray)
        .into_iter()
        .filter_map(|shape| shape.intersect(&ray).map(|distance| (shape, distance)))
        .min_by(|(_, xd), (_, yd)| xd.partial_cmp(yd).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(shape, distance)| color(&ray, &scene, shape, distance))
}

fn color(ray: &Ray, _scene: &Scene, shape: &Box<dyn Shape>, distance: f32) -> Color {
    let hit_point = ray.origin + (ray.direction * distance);
    let normal = shape.normal_at(&hit_point);

    (0.5 * (normal + cgmath::vec3(1.0, 1.0, 1.0))).try_into().unwrap()
}