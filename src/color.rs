use crate::math::*;
use image::Rgb;
use num_traits::NumCast;
use std::convert::{Infallible, TryFrom};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(pub [FloatType; 4]);

impl Color {
    pub fn get_r(&self) -> FloatType {
        self.0[0]
    }
    pub fn get_g(&self) -> FloatType {
        self.0[1]
    }
    pub fn get_b(&self) -> FloatType {
        self.0[2]
    }
    pub fn get_a(&self) -> FloatType {
        self.0[3]
    }

    pub fn gamma(self, power: FloatType) -> Self {
        Self([
            self.0[0].powf(1.0 / power),
            self.0[1].powf(1.0 / power),
            self.0[2].powf(1.0 / power),
            self.0[3],
        ])
    }

    pub fn attenuate(self, attenuation: FloatType) -> Self {
        Self([
            self.0[0] * attenuation,
            self.0[1] * attenuation,
            self.0[2] * attenuation,
            self.0[3],
        ])
    }
}

impl<T: image::Primitive> From<Rgb<T>> for Color {
    fn from(p: Rgb<T>) -> Self {
        let max_t = T::max_value();
        let max_t = max_t.to_f64().unwrap();

        let Rgb(p) = p;
        Color([
            p[0].to_f64().unwrap() / max_t,
            p[1].to_f64().unwrap() / max_t,
            p[2].to_f64().unwrap() / max_t,
            1.0,
        ])
    }
}

impl<T: image::Primitive> From<Color> for Rgb<T> {
    fn from(p: Color) -> Self {
        let max_t = T::max_value();
        let max_t = max_t.to_f64().unwrap();

        Rgb([
            NumCast::from(p.get_r().max(0.0).min(1.0) * max_t).unwrap(),
            NumCast::from(p.get_g().max(0.0).min(1.0) * max_t).unwrap(),
            NumCast::from(p.get_b().max(0.0).min(1.0) * max_t).unwrap(),
        ])
    }
}

impl From<Color> for cgmath::Vector4<FloatType> {
    fn from(p: Color) -> Self {
        cgmath::vec4(p.0[0], p.0[1], p.0[2], p.0[3])
    }
}

fn check_channel(channel: FloatType) -> Result<FloatType, Infallible> {
    Ok(channel)
}

impl TryFrom<cgmath::Vector4<FloatType>> for Color {
    type Error = Infallible;

    fn try_from(p: cgmath::Vector4<FloatType>) -> Result<Self, Self::Error> {
        Ok(Color([
            check_channel(p.x)?,
            check_channel(p.y)?,
            check_channel(p.z)?,
            check_channel(p.w)?,
        ]))
    }
}

impl TryFrom<cgmath::Vector3<FloatType>> for Color {
    type Error = Infallible;

    fn try_from(p: cgmath::Vector3<FloatType>) -> Result<Self, Self::Error> {
        Ok(Color([
            check_channel(p.x)?,
            check_channel(p.y)?,
            check_channel(p.z)?,
            1.0,
        ]))
    }
}

pub mod constants {
    use super::*;

    macro_rules! color_constant {
        ($name:ident, $r:expr, $g:expr, $b:expr) => {
            #[allow(dead_code)]
            pub const $name: Color = Color([$r, $g, $b, 1.0]);
        };
    }

    color_constant!(BLACK, 0.0, 0.0, 0.0);
    color_constant!(RED, 1.0, 0.0, 0.0);
    color_constant!(GREEN, 0.0, 1.0, 0.0);
    color_constant!(BLUE, 0.0, 0.0, 1.0);
    color_constant!(YELLOW, 1.0, 1.0, 0.0);
    color_constant!(MAGENTA, 1.0, 0.0, 1.0);
    color_constant!(CYAN, 0.0, 1.0, 1.0);
    color_constant!(WHITE, 1.0, 1.0, 1.0);
}
