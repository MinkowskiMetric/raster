use crate::color::Color;
use crate::scene::Scene;

use image::{Pixel, SurfaceMut};

use std::convert::TryInto;

use cgmath::prelude::*;
use rand::prelude::*;

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
        let colv = (0..SAMPLE_COUNT)
            .into_iter()
            .map(|_s| {
                (
                    (x as f32) - 0.5 + random::<f32>(),
                    (y as f32) - 0.5 + random::<f32>(),
                )
            })
            .map(|(x, y)| {
                let direction = scene.camera().pixel_to_viewport(x, y);
                let ray = Ray::new(scene.camera().position(), direction);

                cgmath::Vector4::from(trace(&ray, scene, 0))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
        let colv = colv / (SAMPLE_COUNT as f32);
        let col: Color = colv.try_into().unwrap();
        *pixel = col.gamma(2.0).into();
    }
}

const MAX_DEPTH: usize = 50;

pub fn trace(ray: &Ray, scene: &Scene, depth: usize) -> Color {
    scene
        .intersect_shapes(&ray)
        .into_iter()
        .filter_map(|shape| {
            shape
                .intersect(&ray, 0.001, std::f32::INFINITY)
                .map(|distance| (shape, distance))
        })
        .min_by(|(_, xr), (_, yr)| {
            xr.distance
                .partial_cmp(&yr.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, hit_record)| {
            if depth < MAX_DEPTH {
                if let Some(scatter_record) = hit_record.material.scatter(&ray, &hit_record) {
                    let trace_color = trace(&scatter_record.scattered, &scene, depth + 1);

                    scatter_record
                        .attenuation
                        .mul_element_wise(cgmath::Vector4::from(trace_color).truncate())
                        .try_into()
                        .unwrap()
                } else {
                    Color::BLACK
                }
            } else {
                Color::BLACK
            }
        })
        .unwrap_or_else(|| {
            let unit_direction = ray.direction;
            let t = 0.5 * (1.0 - unit_direction.y);
            (((1.0 - t) * cgmath::vec3(1.0, 1.0, 1.0)) + (t * cgmath::vec3(0.5, 0.7, 1.0)))
                .try_into()
                .unwrap()
        })
}
