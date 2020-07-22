use crate::math::*;
use image::{rgba, RgbaPixel};
use std::convert::{Infallible, TryFrom};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color([FloatType; 4]);

macro_rules! color_constant {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        #[allow(dead_code)]
        pub const $name: Color = Color([$r, $g, $b, 1.0]);
    };
}

impl Color {
    color_constant!(BLACK, 0.0, 0.0, 0.0);
    color_constant!(RED, 1.0, 0.0, 0.0);
    color_constant!(GREEN, 0.0, 1.0, 0.0);
    color_constant!(BLUE, 0.0, 0.0, 1.0);
    color_constant!(YELLOW, 1.0, 1.0, 0.0);
    color_constant!(MAGENTA, 1.0, 0.0, 1.0);
    color_constant!(CYAN, 0.0, 1.0, 1.0);
    color_constant!(WHITE, 1.0, 1.0, 1.0);

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

impl From<RgbaPixel> for Color {
    fn from(p: RgbaPixel) -> Self {
        Color([
            FloatType::from(p.get_r()) / 255.0,
            FloatType::from(p.get_g()) / 255.0,
            FloatType::from(p.get_b()) / 255.0,
            FloatType::from(p.get_a()) / 255.0,
        ])
    }
}

impl From<Color> for RgbaPixel {
    fn from(p: Color) -> Self {
        rgba!(
            (p.get_r() * 255.0) as u8,
            (p.get_g() * 255.0) as u8,
            (p.get_b() * 255.0) as u8,
            (p.get_a() * 255.0) as u8
        )
    }
}

impl From<Color> for cgmath::Vector4<FloatType> {
    fn from(p: Color) -> Self {
        cgmath::vec4(p.0[0], p.0[1], p.0[2], p.0[3])
    }
}

fn check_channel(channel: FloatType) -> Result<FloatType, Infallible> {
    if channel >= 0.0 && channel <= 1.0 {
        Ok(channel)
    } else if channel > 1.0 {
        Ok(1.0)
    } else {
        Ok(0.0)
    }
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
