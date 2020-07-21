use cgmath::prelude::*;
use rand::prelude::*;

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

pub fn random_in_unit_disk() -> cgmath::Vector3<f32> {
    loop {
        let p = cgmath::vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.0);
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}
