use image::{rgba, RgbaPixel};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color([f32; 4]);

macro_rules! color_constant {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        #[allow(dead_code)]
        pub const $name: Color = Color([$r, $g, $b, 1.0]);
    };
}

impl Color {
    color_constant!(BLACK, 0.0, 0.0, 0.0);
    color_constant!(RED, 1.0, 0.0, 0.0);
    color_constant!(GREEN, 0.0, 1.0, 0.0);
    color_constant!(BLUE, 0.0, 0.0, 1.0);
    color_constant!(YELLOW, 1.0, 1.0, 0.0);
    color_constant!(MAGENTA, 1.0, 0.0, 1.0);
    color_constant!(CYAN, 0.0, 1.0, 1.0);
    color_constant!(WHITE, 1.0, 1.0, 1.0);

    pub fn get_r(&self) -> f32 {
        self.0[0]
    }
    pub fn get_g(&self) -> f32 {
        self.0[1]
    }
    pub fn get_b(&self) -> f32 {
        self.0[2]
    }
    pub fn get_a(&self) -> f32 {
        self.0[3]
    }

    pub fn gamma(self, power: f32) -> Self {
        Self([
            self.0[0].powf(1.0 / power),
            self.0[1].powf(1.0 / power),
            self.0[2].powf(1.0 / power),
            self.0[3],
        ])
    }
}

impl From<RgbaPixel> for Color {
    fn from(p: RgbaPixel) -> Self {
        Color([
            f32::from(p.get_r()) / 255.0,
            f32::from(p.get_g()) / 255.0,
            f32::from(p.get_b()) / 255.0,
            f32::from(p.get_a()) / 255.0,
        ])
    }
}

impl From<Color> for RgbaPixel {
    fn from(p: Color) -> Self {
        rgba!(
            (p.get_r() * 255.0) as u8,
            (p.get_g() * 255.0) as u8,
            (p.get_b() * 255.0) as u8,
            (p.get_a() * 255.0) as u8
        )
    }
}
