mod dielectric;
mod diffuse_light;
mod lambertian;
mod material;
mod metal;
mod utils;

pub use material::{Material, PartialScatterResult, ScatterResult};

pub mod factories {
    use super::*;

    pub use dielectric::factories::*;
    pub use diffuse_light::factories::*;
    pub use lambertian::factories::*;
    pub use metal::factories::*;
}
