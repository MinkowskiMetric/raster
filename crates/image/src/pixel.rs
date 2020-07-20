use num::traits::Zero;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RgbaPixel(pub [u8; 4]);

pub trait Pixel: Clone {
    type Subpixel: Zero + Clone;

    const CHANNEL_COUNT: usize;

    fn channels(&self) -> &[Self::Subpixel];
    fn channels_mut(&mut self) -> &mut [Self::Subpixel];

    fn from_slice(slice: &[Self::Subpixel]) -> &Self;
    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self;
}

impl Pixel for RgbaPixel {
    type Subpixel = u8;

    const CHANNEL_COUNT: usize = 4;

    fn channels(&self) -> &[Self::Subpixel] {
        &self.0
    }

    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        &mut self.0
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        assert_eq!(slice.len(), 4);
        unsafe { &*(slice.as_ptr() as *const Self) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self {
        assert_eq!(slice.len(), 4);
        unsafe { &mut *(slice.as_ptr() as *mut Self) }
    }
}

#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        $crate::RgbaPixel([$b, $g, $r, 255])
    };
}

#[macro_export]
macro_rules! rgba {
    ($r:expr,  $g:expr, $b:expr, $a:expr) => {
        $crate::RgbaPixel([$b, $g, $r, $a])
    };
}

macro_rules! rgb_constant {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        #[allow(dead_code)]
        pub const $name: RgbaPixel = rgb!($r, $g, $b);
    };
}

impl RgbaPixel {
    rgb_constant!(BLACK, 0, 0, 0);
    rgb_constant!(RED, 255, 0, 0);
    rgb_constant!(GREEN, 0, 255, 0);
    rgb_constant!(BLUE, 0, 0, 255);
    rgb_constant!(YELLOW, 255, 255, 0);
    rgb_constant!(MAGENTA, 255, 0, 255);
    rgb_constant!(CYAN, 0, 255, 255);
    rgb_constant!(WHITE, 255, 255, 255);

    pub fn get_r(&self) -> u8 {
        self.0[2]
    }

    pub fn get_g(&self) -> u8 {
        self.0[1]
    }

    pub fn get_b(&self) -> u8 {
        self.0[0]
    }

    pub fn get_a(&self) -> u8 {
        self.0[3]
    }
}
