use crate::math::*;
use random_fast_rng::{local_rng, Random};

use std::convert::TryInto;

pub fn random_in_range(min: FloatType, max: FloatType) -> FloatType {
    (local_rng().gen::<FloatType>() * (max - min)) + min
}

pub fn random_int_in_range(min: i32, max: i32) -> i32 {
    random_in_range(min as f32, max as f32) as i32
}

pub fn random_in_unit_sphere() -> Vector3 {
    loop {
        let p = vec3(
            random_in_range(-1.0, 1.0),
            random_in_range(-1.0, 1.0),
            random_in_range(-1.0, 1.0),
        );
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

pub fn random_unit_vector() -> Vector3 {
    let a = random_in_range(0.0, 2.0 * constants::PI);
    let z = random_in_range(-1.0, 1.0);
    let r = (1.0 - z * z).sqrt();
    vec3(r * a.cos(), r * a.sin(), z)
}

pub fn random_in_unit_disk() -> Vector3 {
    loop {
        let p = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.0);
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

pub fn random_color_in_range(min: FloatType, max: FloatType) -> crate::color::Color {
    vec3(
        random_in_range(min, max),
        random_in_range(min, max),
        random_in_range(min, max),
    )
    .try_into()
    .unwrap()
}
