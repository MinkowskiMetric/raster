use crate::camera::Camera;
use crate::ray_scanner::Ray;

use image::RgbaPixel;

pub struct AlignedBoundingBox {
    bounds: [cgmath::Vector3<f32>; 2],
}

impl AlignedBoundingBox {
    pub fn from_center_and_size(center: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>) -> Self {
        // Ensure that vmax - vmin is always positive or zero
        let half_size = cgmath::vec3(size.x.abs() / 2.0, size.y.abs() / 2.0, size.z.abs() / 2.0);

        AlignedBoundingBox {
            bounds: [center - half_size, center + half_size],
        }
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        let mut tmin = (self.bounds[ray.sign[0]].x - ray.origin.x) * ray.inv_direction.x;
        let mut tmax = (self.bounds[1 - ray.sign[0]].x - ray.origin.x) * ray.inv_direction.x;
        let tymin = (self.bounds[ray.sign[1]].y - ray.origin.y) * ray.inv_direction.y;
        let tymax = (self.bounds[1 - ray.sign[1]].y - ray.origin.y) * ray.inv_direction.y;

        if (tmin > tymax) || (tymin > tmax) {
            false
        } else {
            if tymin > tmin {
                tmin = tymin;
            }
            if tymax < tmax {
                tmax = tymax;
            }

            let tzmin = (self.bounds[ray.sign[2]].z - ray.origin.z) * ray.inv_direction.z;
            let tzmax = (self.bounds[1 - ray.sign[2]].z - ray.origin.z) * ray.inv_direction.z;

            if (tmin > tzmax) || (tzmin > tmax) {
                false
            } else {
                true
            }
        }
    }
}

pub trait Shape {
    fn bounding_box(&self) -> &AlignedBoundingBox;
    fn intersect(&self, ray: &Ray) -> Option<f32>;
    fn color(&self) -> RgbaPixel;
}

pub struct Scene {
    camera: Camera,
    shapes: Box<[Box<dyn Shape>]>,
}

impl Scene {
    pub fn new(camera: Camera, shapes: Vec<Box<dyn Shape>>) -> Self {
        let shapes = shapes.into_boxed_slice();

        Scene { camera, shapes }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn shapes(&self) -> &[Box<dyn Shape>] {
        &self.shapes
    }

    pub fn intersect_shapes(&self, ray: &Ray) -> Vec<&Box<dyn Shape>> {
        self.shapes()
            .iter()
            .filter(|shape| shape.bounding_box().intersects(ray))
            .collect()
    }
}
