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

#[repr(C, align(32))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct M256Point3 {
    val: [f64; 4],
}

impl From<Point3> for M256Point3 {
    fn from(p: Point3) -> Self {
        Self {
            val: [1.0, p.z, p.y, p.x],
        }
    }
}

impl From<M256Point3> for Point3 {
    fn from(p: M256Point3) -> Self {
        Self::new(p.x(), p.y(), p.z())
    }
}

impl M256Point3 {
    #[target_feature(enable = "avx")]
    pub unsafe fn load_v(&self) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;
        _mm256_load_pd(&self.val[0])
    }

    pub fn into_point(self) -> Point3 {
        self.into()
    }

    pub fn x(&self) -> f64 {
        self.val[3]
    }

    pub fn y(&self) -> f64 {
        self.val[2]
    }

    pub fn z(&self) -> f64 {
        self.val[1]
    }
}

#[repr(C, align(32))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct M256Vector3 {
    val: [f64; 4],
}

impl From<Vector3> for M256Vector3 {
    fn from(p: Vector3) -> Self {
        Self {
            val: [1.0, p.z, p.y, p.x],
        }
    }
}

impl From<M256Vector3> for Vector3 {
    fn from(p: M256Vector3) -> Self {
        Self::new(p.x(), p.y(), p.z())
    }
}

impl M256Vector3 {
    #[target_feature(enable = "avx")]
    pub unsafe fn load_v(&self) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;
        _mm256_load_pd(&self.val[0])
    }

    pub fn into_vector(self) -> Vector3 {
        self.into()
    }

    pub fn x(&self) -> f64 {
        self.val[3]
    }

    pub fn y(&self) -> f64 {
        self.val[2]
    }

    pub fn z(&self) -> f64 {
        self.val[1]
    }
}
