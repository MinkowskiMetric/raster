use crate::{
    pixel::{Pixel, RgbaPixel},
    surface::Surface,
};

pub trait Encoder<P>
where
    P: Pixel,
{
    fn encode(self, image: &impl Surface<P>) -> Vec<u8>;
}

pub struct BmpEncoder();

impl BmpEncoder {
    pub fn new() -> Self {
        Self {}
    }

    fn bmp_size_in_bytes(width: usize, height: usize) -> usize {
        14 + 40 + Self::image_size_in_bytes(width, height)
    }

    fn image_size_in_bytes(width: usize, height: usize) -> usize {
        width * height * 3
    }

    fn generate_bitmap_file_header(width: usize, height: usize) -> [u8; 14] {
        let mut ret = [0u8; 14];

        ret[0] = b'B';
        ret[1] = b'M';
        ret[2..6].copy_from_slice(&(Self::bmp_size_in_bytes(width, height) as u32).to_le_bytes());
        ret[6..10].copy_from_slice(&0u32.to_le_bytes());
        ret[10..14].copy_from_slice(&54u32.to_le_bytes());

        ret
    }

    fn generate_bitmap_info_header(width: usize, height: usize) -> [u8; 40] {
        let mut ret = [0u8; 40];

        ret[0..4].copy_from_slice(&40u32.to_le_bytes());
        ret[4..8].copy_from_slice(&((width as u32).to_le_bytes()));
        ret[8..12].copy_from_slice(&((height as u32).to_le_bytes()));
        ret[12..14].copy_from_slice(&1u16.to_le_bytes());
        ret[14..16].copy_from_slice(&24u16.to_le_bytes());
        ret[16..20].copy_from_slice(&0u32.to_le_bytes());
        ret[20..24].copy_from_slice(&0u32.to_le_bytes());
        ret[24..28].copy_from_slice(&0u32.to_le_bytes());
        ret[28..32].copy_from_slice(&0u32.to_le_bytes());
        ret[32..36].copy_from_slice(&0u32.to_le_bytes());
        ret[36..40].copy_from_slice(&0u32.to_le_bytes());

        ret
    }
}

impl<P> Encoder<P> for BmpEncoder
where
    P: Pixel + Into<RgbaPixel>,
{
    fn encode(self, image: &impl Surface<P>) -> Vec<u8> {
        let (width, height) = image.dimensions();

        let mut ret = vec![0u8; Self::bmp_size_in_bytes(width, height)];

        ret[0..14].copy_from_slice(&Self::generate_bitmap_file_header(width, height));
        ret[14..54].copy_from_slice(&Self::generate_bitmap_info_header(width, height));

        ret[54..]
            .chunks_mut(3)
            .zip(image.pixels())
            .fold({}, |_, (dst, src)| {
                let src: RgbaPixel = src.clone().into();
                dst.copy_from_slice(&src.channels()[0..3]);
            });

        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        rgba,
        surface::{filled_image, SurfaceMut, VecImageBuffer},
    };

    fn test_image() -> Option<VecImageBuffer<RgbaPixel>> {
        if let Some(mut img) = filled_image(10, 10, rgba!(10, 10, 10, 10)) {
            for i in 0..10 {
                img.put_pixel(i, i, rgba!(255, 255, 255, 255));
            }
            Some(img)
        } else {
            None
        }
    }

    #[test]
    fn write_bitmap() {
        let image = test_image()
            .map(|img| BmpEncoder::new().encode(&img))
            .unwrap_or(vec![]);

        assert_eq!(image.len(), 354);
        assert_eq!(image[0], b'B');
    }
}
