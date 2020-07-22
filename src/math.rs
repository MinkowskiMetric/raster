pub type FloatType = f64;
pub type Point3 = cgmath::Point3<FloatType>;
pub type Vector3 = cgmath::Vector3<FloatType>;
pub use cgmath::Deg;
pub use cgmath::Rad;

pub use cgmath::prelude::*;

pub fn vec3<T>(x: T, y: T, z: T) -> cgmath::Vector3<T> {
    cgmath::vec3(x, y, z)
}

trait MyConstants {
    const INFINITY: Self;
    const PI: Self;
}

impl MyConstants for f32 {
    const INFINITY: Self = std::f32::INFINITY;
    const PI: Self = std::f32::consts::PI;
}

impl MyConstants for f64 {
    const INFINITY: Self = std::f64::INFINITY;
    const PI: Self = std::f64::consts::PI;
}

pub mod constants {
    use super::MyConstants;

    pub const INFINITY: super::FloatType = super::FloatType::INFINITY;
    pub const PI: super::FloatType = super::FloatType::PI;
}
