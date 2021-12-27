use crate::{constants, math::*, Color, Ray};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub enum Sky {
    RegularSky,
    ColorSky(Color),
}

impl Sky {
    pub fn sample(&self, ray: &Ray) -> Color {
        match self {
            Sky::RegularSky => {
                let unit_direction = ray.direction;
                let t = 0.5 * (1.0 - unit_direction.y);
                (((1.0 - t) * vec3(1.0, 1.0, 1.0)) + (t * vec3(0.5, 0.7, 1.0)))
                    .try_into()
                    .unwrap()
            }
            Sky::ColorSky(color) => *color,
        }
    }
}

pub mod factories {
    use super::*;

    pub fn regular_sky() -> Sky {
        Sky::RegularSky
    }

    pub fn color_sky(color: Color) -> Sky {
        Sky::ColorSky(color)
    }

    pub fn black_sky() -> Sky {
        color_sky(constants::BLACK)
    }
}
