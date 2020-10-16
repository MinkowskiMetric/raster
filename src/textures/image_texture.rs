use crate::math::*;
use crate::{Color, Texture};
use image::{GenericImageView, Pixel};

#[derive(Clone)]
pub struct ImageTexture<Image: GenericImageView + Sync + Send>(Image);

impl<Image: GenericImageView + Sync + Send> ImageTexture<Image> {
    pub fn new(image: Image) -> Self {
        Self(image)
    }

    fn image(&self) -> &Image {
        &self.0
    }
}

impl<Image: GenericImageView + Sync + Send> Texture for ImageTexture<Image> {
    fn value(&self, _p: Point3, (u, v): (FloatType, FloatType)) -> Color {
        let u = u.max(0.0).min(1.0);
        let v = 1.0 - v.max(0.0).min(1.0);

        let i = ((u * (self.image().width() as FloatType)) as u32).min(self.image().width() - 1);
        let j = ((v * (self.image().height() as FloatType)) as u32).min(self.image().height() - 1);

        self.image().get_pixel(i, j).to_rgb().into()
    }
}

impl<Image: GenericImageView + Sync + Send> std::fmt::Debug for ImageTexture<Image> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageTexture").finish()
    }
}

pub mod factories {
    use super::*;

    pub fn image_texture<Image: image::GenericImageView + Sync + Send>(
        image: Image,
    ) -> ImageTexture<Image> {
        ImageTexture::new(image)
    }
}
