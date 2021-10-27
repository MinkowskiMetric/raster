mod bump_mapper;
mod debug_material;
mod dielectric;
mod diffuse_light;
mod invert_normal;
mod lambertian;
mod material;
mod metal;
mod surface_mapper;
mod utils;

pub use material::{BaseMaterial, Material, PartialScatterResult, ScatterResult};
pub use surface_mapper::SurfaceMapper;

pub mod factories {
    use super::*;

    pub use bump_mapper::factories::*;
    pub use debug_material::factories::*;
    pub use dielectric::factories::*;
    pub use diffuse_light::factories::*;
    pub use invert_normal::factories::*;
    pub use lambertian::factories::*;
    pub use metal::factories::*;
    pub use surface_mapper::factories::*;
}
