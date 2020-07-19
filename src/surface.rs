use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};
use std::slice::{Chunks, ChunksMut};
use num::traits::Zero;
use crate::pixel::Pixel;

pub struct Pixels<'a, P: Pixel + 'a> where P::Subpixel: 'a,
{
    chunks: Chunks<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> Iterator for Pixels<'a, P> where P::Subpixel: 'a,
{
    type Item = &'a P;

    fn next(&mut self) -> Option<&'a P> {
        self.chunks.next().map(|v| <P as Pixel>::from_slice(v))
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for Pixels<'a, P> where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for Pixels<'a, P> where P::Subpixel: 'a,
{
    fn next_back(&mut self) -> Option<&'a P> {
        self.chunks.next_back().map(|v| <P as Pixel>::from_slice(v))
    }
}

pub struct PixelsMut<'a, P: Pixel + 'a> where P::Subpixel: 'a,
{
    chunks: ChunksMut<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> Iterator for PixelsMut<'a, P> where P::Subpixel: 'a,
{
    type Item = &'a mut P;

    fn next(&mut self) -> Option<&'a mut P> {
        self.chunks.next().map(|v| <P as Pixel>::from_slice_mut(v))
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for PixelsMut<'a, P> where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for PixelsMut<'a, P> where P::Subpixel: 'a,
{
    fn next_back(&mut self) -> Option<&'a mut P> {
        self.chunks.next_back().map(|v| <P as Pixel>::from_slice_mut(v))
    }
}

pub struct Rows<'a, P: Pixel + 'a> 
where P::Subpixel: 'a,
{
    chunks: Chunks<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> Iterator for Rows<'a, P>
where P::Subpixel: 'a,
{
    type Item = Pixels<'a, P>;

    fn next(&mut self) -> Option<Pixels<'a, P>> {
        self.chunks.next().map(|row| Pixels { chunks: row.chunks(<P as Pixel>::CHANNEL_COUNT as usize) })
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for Rows<'a, P>
where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for Rows<'a, P>
where P::Subpixel: 'a,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Pixels<'a, P>> {
        self.chunks.next_back().map(|row| Pixels { chunks: row.chunks(<P as Pixel>::CHANNEL_COUNT as usize) })
    }
}

pub struct RowsMut<'a, P: Pixel + 'a> 
where P::Subpixel: 'a,
{
    chunks: ChunksMut<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> Iterator for RowsMut<'a, P>
where P::Subpixel: 'a,
{
    type Item = PixelsMut<'a, P>;

    fn next(&mut self) -> Option<PixelsMut<'a, P>> {
        self.chunks.next().map(|row| PixelsMut { chunks: row.chunks_mut(<P as Pixel>::CHANNEL_COUNT as usize) })
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for RowsMut<'a, P>
where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for RowsMut<'a, P>
where P::Subpixel: 'a,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<PixelsMut<'a, P>> {
        self.chunks.next_back().map(|row| PixelsMut { chunks: row.chunks_mut(<P as Pixel>::CHANNEL_COUNT as usize) })
    }
}

pub struct EnumeratePixels<'a, P: Pixel + 'a>
where P::Subpixel: 'a,
{
    pixels: Pixels<'a, P>,
    x: usize,
    y: usize,
    width: usize,
}

impl<'a, P: Pixel + 'a> Iterator for EnumeratePixels<'a, P>
where P::Subpixel: 'a,
{
    type Item = (usize, usize, &'a P);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        self.pixels.next().map(|p| (x, y, p))
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for EnumeratePixels<'a, P>
where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.pixels.len()
    }
}

pub struct EnumeratePixelsMut<'a, P: Pixel + 'a>
where P::Subpixel: 'a,
{
    pixels: PixelsMut<'a, P>,
    x: usize,
    y: usize,
    width: usize,
}

impl<'a, P: Pixel + 'a> Iterator for EnumeratePixelsMut<'a, P>
where P::Subpixel: 'a,
{
    type Item = (usize, usize, &'a mut P);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        self.pixels.next().map(|p| (x, y, p))
    }
}

impl<'a, P: Pixel + 'a> ExactSizeIterator for EnumeratePixelsMut<'a, P>
where P::Subpixel: 'a,
{
    fn len(&self) -> usize {
        self.pixels.len()
    }
}

pub trait Surface<P: Pixel> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn dimensions(&self) -> (usize, usize);
    fn get_pixel(&self, x: usize, y: usize) -> &P;
    fn pixels(&self) -> Pixels<P>;
    fn rows(&self) -> Rows<P>;
    fn enumerate_pixels(&self) -> EnumeratePixels<P>;
}

pub trait SurfaceMut<P: Pixel>: Surface<P> {
    fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut P;
    fn put_pixel(&mut self, x: usize, y: usize, p: P);
    fn pixels_mut(&mut self) -> PixelsMut<P>;
    fn rows_mut(&mut self) -> RowsMut<P>;
    fn enumerate_pixels_mut(&mut self) -> EnumeratePixelsMut<P>;
    fn clear(&mut self, fill_color: P);
}

pub struct ImageBuffer<P: Pixel, Container> {
    width: usize,
    height: usize,
    _phantom: PhantomData<P>,
    data: Container,
}

impl<P, Container> ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    pub fn from_raw(width: usize, height: usize, data: Container) -> Option<Self> {
        if Self::check_image_fits(width, height, data.len()) {
            Some(ImageBuffer {
                width,
                height,
                data,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn into_raw(self) -> Container {
        self.data
    }

    fn check_image_fits(width: usize, height: usize, len: usize) -> bool {
        let checked_len = Self::image_buffer_len(width, height);
        checked_len.map(|min_len| min_len <= len).unwrap_or(false)
    }

    fn image_buffer_len(width: usize, height: usize) -> Option<usize> {
        Some(<P as Pixel>::CHANNEL_COUNT)
            .and_then(|size| size.checked_mul(width as usize))
            .and_then(|size| size.checked_mul(height as usize))
    }

    fn pixel_range(&self, x: usize, y: usize) -> Option<Range<usize>> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.pixel_range_unchecked(x, y))
        }
    }

    fn pixel_range_unchecked(&self, x: usize, y: usize) -> Range<usize> {
        let channel_count = P::CHANNEL_COUNT;
        let min_index = ((y * self.width) + x) * channel_count;
        min_index..min_index+channel_count
    }
}

impl<P, Container> Surface<P> for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]> + Deref,
{
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_pixel(&self, x: usize, y: usize) -> &P {
        if let Some(range) = self.pixel_range(x, y) {
            P::from_slice(&self.data[range])
        } else {
            panic!("Point ({}, {}) is not in range", x, y);
        }
    }

    fn pixels(&self) -> Pixels<P> {
        Pixels { chunks: self.data.chunks(<P as Pixel>::CHANNEL_COUNT as usize) }
    }

    fn enumerate_pixels(&self) -> EnumeratePixels<P> {
        EnumeratePixels { pixels: self.pixels(), x: 0, y: 0, width: self.width }
    }

    fn rows(&self) -> Rows<P> {
        Rows { chunks: self.data.chunks(<P as Pixel>::CHANNEL_COUNT * self.width), }
    }
}

impl<P, Container> SurfaceMut<P> for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]> + DerefMut,
{
    fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut P {
        if let Some(range) = self.pixel_range(x, y) {
            P::from_slice_mut(&mut self.data[range])
        } else {
            panic!("Point ({}, {}) is not in range", x, y);
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, p: P) {
        *self.get_pixel_mut(x, y) = p;
    }

    fn pixels_mut(&mut self) -> PixelsMut<P> {
        PixelsMut { chunks: self.data.chunks_mut(<P as Pixel>::CHANNEL_COUNT as usize) }
    }

    fn rows_mut(&mut self) -> RowsMut<P> {
        RowsMut { chunks: self.data.chunks_mut(<P as Pixel>::CHANNEL_COUNT * self.width), }
    }

    fn enumerate_pixels_mut(&mut self) -> EnumeratePixelsMut<P> {
        let width = self.width;
        EnumeratePixelsMut { pixels: self.pixels_mut(), x: 0, y: 0, width }
    }

    fn clear(&mut self, fill_color: P) {
        for p in self.pixels_mut() {
            *p = fill_color.clone();
        }
    }
}

pub type VecImageBuffer<P> = ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>;

fn sized_image<P: Pixel>(width: usize, height: usize) -> Option<VecImageBuffer<P>> {
    VecImageBuffer::<P>::image_buffer_len(width, height)
        .and_then(|bytes| VecImageBuffer::<P>::from_raw(width, height, vec![P::Subpixel::zero(); bytes]) )
}

pub fn filled_image<P: Pixel>(width: usize, height: usize, fill: P) -> Option<VecImageBuffer<P>> {
    if let Some(mut img) = sized_image(width, height) {
        img.clear(fill);
        Some(img)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pixel::RgbaPixel;

    #[test]
    fn create_filled_buffer() {
        let color = RgbaPixel::new(1,2,3,4);
        let buffer = filled_image(10, 10, color.clone()).unwrap();

        assert_eq!(10, buffer.width());
        assert_eq!(10, buffer.height());
        assert_eq!((10, 10), buffer.dimensions());
        for p in buffer.pixels() {
            assert_eq!(*p, color);
        }
        assert_eq!(*buffer.get_pixel(0, 0), color);
    }
}