use crate::{math::*, Transformable};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
    pub time: FloatType,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3, time: FloatType) -> Self {
        Self {
            origin,
            direction,
            time,
        }
    }
}

impl Transformable for Ray {
    type Target = Ray;

    fn core_transform(self, _transform: &Matrix4, inverse_transform: &Matrix4) -> Self::Target {
        Self {
            origin: inverse_transform.transform_point(self.origin),
            direction: inverse_transform.transform_vector(self.direction),
            time: self.time,
        }
    }
}
