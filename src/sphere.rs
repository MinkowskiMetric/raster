use crate::scene::{Shape, AlignedBoundingBox};
use crate::pixel::RgbaPixel;
use crate::ray_scanner::Ray;
use cgmath::prelude::*;

pub struct Sphere {
    center: cgmath::Vector3<f32>,
    radius: f32,
    color: RgbaPixel,
    bounding_box: AlignedBoundingBox,
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32, color: RgbaPixel) -> Self {
        Self {
            center,
            radius,
            color,
            bounding_box: AlignedBoundingBox::from_center_and_size(center, cgmath::vec3(radius * 2.0, radius * 2.0, radius * 2.0)),
        }
    }
}

impl Shape for Sphere {
    fn bounding_box(&self) -> &AlignedBoundingBox {
        &self.bounding_box
    }

    fn color(&self) -> RgbaPixel {
        self.color
    }

    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let l = self.center - ray.origin;
        let tca  = l.dot(ray.direction);
        if tca < 0.0 {
            None
        } else {
            let d2 = l.dot(l) - (tca * tca);
            let radius2 = self.radius * self.radius;
            if d2 > radius2 {
                None
            } else {
                let thc = (radius2 - d2).sqrt();
                let t0 = tca - thc;
                let t1 = tca + thc;
                
                if t0 < 0.0 {
                    Some(t1)
                } else {
                    Some(t0)
                }
            }
        }
    }
}