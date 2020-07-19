use crate::{
    encoder::{Encoder},
    pixel::Pixel,
    surface::Surface,
};

pub trait ImageWriter<P: Pixel>: Sized {
    fn write_image(self, image: &impl Surface<P>, write: &mut impl std::io::Write) -> std::io::Result<()>;

    fn write_image_to_file(self, image: &impl Surface<P>, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        self.write_image(image, &mut f)
    }
}

impl<P: Pixel, T: Encoder<P>> ImageWriter<P> for T {
    fn write_image(self, image: &impl Surface<P>, write: &mut impl std::io::Write) -> std::io::Result<()> {
        write.write(&self.encode(image)).map(|_| ())
    }
}