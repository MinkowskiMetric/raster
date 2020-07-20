use crate::color::Color;
use crate::scene::{Scene};

use image::{Pixel, SurfaceMut};

use std::convert::TryInto;

use rand::prelude::*;
use cgmath::prelude::*;

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

const SAMPLE_COUNT: usize = 100;

pub fn scan<P: Pixel + From<Color>>(image: &mut impl SurfaceMut<P>, scene: &Scene) {
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let colv = (0..SAMPLE_COUNT).into_iter()
            .map(|_s| ((x as f32) - 0.5 + random::<f32>(), (y as f32) - 0.5 + random::<f32>()))
            .map(|(x, y)| {
                let direction = scene.camera().pixel_to_viewport(x, y);
                let ray = Ray::new(scene.camera().position(), direction);

                cgmath::Vector4::from(trace(&ray, scene))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
        let colv = colv / (SAMPLE_COUNT as f32);
        let col: Color = colv.try_into().unwrap();
        *pixel = col.gamma(2.0).into();
    }
}

fn random_in_unit_sphere() -> cgmath::Vector3<f32> {
    loop {
        let p = cgmath::vec3(random::<f32>(), random::<f32>(), random::<f32>());
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}
pub fn trace(ray: &Ray, scene: &Scene) -> Color {
    scene
        .intersect_shapes(&ray)
        .into_iter()
        .filter_map(|shape| shape.intersect(&ray).map(|distance| (shape, distance)))
        .min_by(|(_, xd), (_, yd)| xd.partial_cmp(yd).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(shape, distance)| {
            let hit_point = ray.origin + (ray.direction * distance);
            let normal = shape.normal_at(&hit_point);

            /*(0.5 * cgmath::vec3(normal.x + 1.0, normal.y + 1.0, 1.0 - normal.z)).try_into().unwrap()*/

            let target = hit_point + normal + random_in_unit_sphere();
            let test_direction = (target - hit_point).normalize();
            let test_ray = Ray::new(hit_point, test_direction);
            let projected_color = trace(&test_ray, &scene);
            let projected_color = cgmath::vec3(projected_color.get_r(), projected_color.get_g(), projected_color.get_b());
            let object_color = cgmath::vec3(shape.color().get_r(), shape.color().get_g(), shape.color().get_b());
            let object_color = cgmath::vec3(projected_color.x * object_color.x, projected_color.y * object_color.y, projected_color.z * object_color.z);

            (0.5 * object_color).try_into().unwrap()
        })
        .unwrap_or_else(|| {
            let unit_direction = ray.direction;
            let t = 0.5 * (1.0 - unit_direction.y);
            (((1.0 - t) * cgmath::vec3(1.0, 1.0, 1.0)) + (t * cgmath::vec3(0.5, 0.7, 1.0))).try_into().unwrap()
        })
}