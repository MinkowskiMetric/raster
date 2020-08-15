mod checker_texture;
mod image_texture;
mod noise_normal;
mod noise_texture;
mod solid_texture;
mod texture;

pub use texture::Texture;

pub mod factories {
    use super::*;

    pub use checker_texture::factories::*;
    pub use image_texture::factories::*;
    pub use noise_normal::factories::*;
    pub use noise_texture::factories::*;
    pub use solid_texture::factories::*;
}
