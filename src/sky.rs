use crate::color::Color;
use crate::constants;
use crate::math::*;
use crate::ray_scanner::Ray;
use std::convert::TryInto;
use std::sync::Arc;

pub trait Sky: Sync + Send + std::fmt::Debug {
    fn sample(&self, ray: &Ray) -> Color;
}

pub type SharedSky = Arc<dyn Sky>;

#[derive(Debug)]
pub struct RegularSky();

impl RegularSky {
    pub fn new() -> Self {
        Self()
    }
}

impl Sky for RegularSky {
    fn sample(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction;
        let t = 0.5 * (1.0 - unit_direction.y());
        (((1.0 - t) * vec3(1.0, 1.0, 1.0)) + (t * vec3(0.5, 0.7, 1.0)))
            .try_into()
            .unwrap()
    }
}

#[derive(Debug)]
pub struct ColorSky(Color);

impl ColorSky {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    pub fn color(&self) -> Color {
        self.0
    }
}

impl Sky for ColorSky {
    fn sample(&self, _rey: &Ray) -> Color {
        self.color()
    }
}

pub mod factories {
    use super::*;

    pub fn regular_sky() -> Arc<RegularSky> {
        Arc::new(RegularSky::new())
    }

    pub fn color_sky(color: Color) -> Arc<ColorSky> {
        Arc::new(ColorSky::new(color))
    }

    pub fn black_sky() -> Arc<ColorSky> {
        color_sky(constants::BLACK)
    }
}
