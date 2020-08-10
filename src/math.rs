pub type FloatType = f64;
pub type Point3 = cgmath::Point3<FloatType>;
pub type Vector3 = cgmath::Vector3<FloatType>;
pub type Vector4 = cgmath::Vector4<FloatType>;
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
    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn load_v(&self) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;
        _mm256_load_pd(&self.val[0])
    }

    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn from_v(v: std::arch::x86_64::__m256d) -> Self {
        use std::arch::x86_64::*;

        // There is almost certainly a way to use maybeuninit here to avoid zeroing the
        // array at this point
        let mut ret = Self { val: [0.0; 4] };
        _mm256_store_pd(&mut ret.val[0], v);
        ret
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

    pub fn w(&self) -> f64 {
        self.val[0]
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
            val: [0.0, p.z, p.y, p.x],
        }
    }
}

impl From<M256Vector3> for Vector3 {
    fn from(p: M256Vector3) -> Self {
        Self::new(p.x(), p.y(), p.z())
    }
}

impl From<Vector4> for M256Vector3 {
    fn from(p: Vector4) -> Self {
        Self {
            val: [p.w, p.z, p.y, p.x],
        }
    }
}

impl From<M256Vector3> for Vector4 {
    fn from(p: M256Vector3) -> Self {
        Self::new(p.x(), p.y(), p.z(), p.w())
    }
}

impl M256Vector3 {
    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn load_v(&self) -> std::arch::x86_64::__m256d {
        use std::arch::x86_64::*;
        _mm256_load_pd(&self.val[0])
    }

    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn from_v(v: std::arch::x86_64::__m256d) -> Self {
        use std::arch::x86_64::*;

        // There is almost certainly a way to use maybeuninit here to avoid zeroing the
        // array at this point
        let mut ret = Self { val: [0.0; 4] };
        _mm256_store_pd(&mut ret.val[0], v);
        ret
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

    pub fn w(&self) -> f64 {
        self.val[0]
    }
}

#[inline]
#[target_feature(enable = "avx")]
pub unsafe fn _mm256_dot_pd(x: std::arch::x86_64::__m256d, y: std::arch::x86_64::__m256d) -> f64 {
    use std::arch::x86_64::*;

    let xy = _mm256_mul_pd(x, y);
    let xy_l = _mm256_castpd256_pd128(xy);
    let xy_h = _mm256_extractf128_pd(xy, 1);
    let xy_l = _mm_add_pd(xy_l, xy_h);
    let xy_h = _mm_unpackhi_pd(xy_l, xy_l);

    _mm_cvtsd_f64(_mm_add_pd(xy_l, xy_h))
}

pub fn trilinear_interp(c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let mut accum = 0.0;
    for i in 0..2 {
        let fi = i as f64;
        for j in 0..2 {
            let fj = j as f64;
            for k in 0..2 {
                let fk = k as f64;

                accum = accum
                    + (((fi * u) + ((1.0 - fi) * (1.0 - u)))
                        * ((fj * v) + ((1.0 - fj) * (1.0 - v)))
                        * ((fk * w) + ((1.0 - fk) * (1.0 - w)))
                        * c[i][j][k]);
            }
        }
    }
    accum
}
