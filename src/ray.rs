use std::simd::{f32x4, Simd};

use crate::{math::*, Transformable};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
    origin: f32x4,
    direction: f32x4,
    time: FloatType,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3, time: FloatType) -> Self {
        Self {
            origin: Simd::from_array([origin.x, origin.y, origin.z, 1.0]),
            direction: Simd::from_array([direction.x, direction.y, direction.z, 0.0]),
            time,
        }
    }

    pub fn origin(&self) -> Point3 {
        let origin_array = self.origin.as_array();
        debug_assert_eq!(origin_array[3], 1.0);
        point3(origin_array[0], origin_array[1], origin_array[2])
    }

    pub fn origin_v(&self) -> f32x4 {
        self.origin
    }

    pub fn direction(&self) -> Vector3 {
        let direction_array = self.direction.as_array();
        debug_assert_eq!(direction_array[3], 0.0);
        vec3(direction_array[0], direction_array[1], direction_array[2])
    }

    pub fn direction_v(&self) -> f32x4 {
        self.direction
    }

    pub fn time(&self) -> FloatType {
        self.time
    }
}

impl Transformable for Ray {
    type Target = Ray;

    fn core_transform(self, _transform: &Matrix4, inverse_transform: &Matrix4) -> Self::Target {
        Self::new(
            inverse_transform.transform_point(self.origin()),
            inverse_transform.transform_vector(self.direction()),
            self.time(),
        )
    }
}
