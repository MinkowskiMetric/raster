mod encoder;
#[macro_use]
mod pixel;
mod surface;
mod writer;

pub use encoder::BmpEncoder;
pub use pixel::{Pixel, RgbaPixel};
pub use surface::{filled_image, Surface, SurfaceMut};
pub use writer::ImageWriter;

pub mod prelude {
    pub use crate::surface::{Surface, SurfaceMut};
    pub use crate::writer::ImageWriter;
}
