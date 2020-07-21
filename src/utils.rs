use cgmath::prelude::*;
use rand::prelude::*;

use std::convert::TryInto;

pub fn random_in_range(min: f32, max: f32) -> f32 {
    (random::<f32>() * (max - min)) + min
}

pub fn random_in_unit_sphere() -> cgmath::Vector3<f32> {
    loop {
        let p = cgmath::vec3(
            random_in_range(-1.0, 1.0),
            random_in_range(-1.0, 1.0),
            random_in_range(-1.0, 1.0),
        );
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

pub fn random_unit_vector() -> cgmath::Vector3<f32> {
    let a = random_in_range(0.0, 2.0 * std::f32::consts::PI);
    let z = random_in_range(-1.0, 1.0);
    let r = (1.0 - z * z).sqrt();
    cgmath::vec3(r * a.cos(), r * a.sin(), z)
}

pub fn random_in_unit_disk() -> cgmath::Vector3<f32> {
    loop {
        let p = cgmath::vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.0);
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

pub fn random_color_in_range(min: f32, max: f32) -> crate::color::Color {
    cgmath::vec3(random_in_range(min, max), random_in_range(min, max), random_in_range(min, max)).try_into().unwrap()
}