use crate::scene::Scene;

use image::{Pixel, RgbaPixel, SurfaceMut};

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

pub fn scan<P: Pixel + From<RgbaPixel>>(image: &mut impl SurfaceMut<P>, scene: &Scene) {
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let direction = scene.camera().pixel_to_viewport(x, y);

        *pixel = trace(Ray::new(scene.camera().position(), direction), scene)
            .unwrap_or(RgbaPixel::WHITE)
            .into();
    }
}

pub fn trace(ray: Ray, scene: &Scene) -> Option<RgbaPixel> {
    scene
        .intersect_shapes(&ray)
        .into_iter()
        .filter_map(|shape| shape.intersect(&ray).map(|distance| (shape, distance)))
        .min_by(|(_, xd), (_, yd)| xd.partial_cmp(yd).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(shape, _)| shape.color())
}
