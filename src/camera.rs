use cgmath::prelude::*;

pub struct Camera {
    position: cgmath::Vector3<f32>,
    inv_width: f32,
    inv_height: f32,
    aspect_ratio: f32,
    angle: f32,
}

impl Camera {
    pub fn new(
        position: cgmath::Vector3<f32>,
        width: usize,
        height: usize,
        fov: cgmath::Rad<f32>,
    ) -> Self {
        let inv_width = 1.0_f32 / (width as f32);
        let inv_height = 1.0_f32 / (height as f32);
        let aspect_ratio = inv_height / inv_width;

        let angle = (fov / 2.0).tan();

        Self {
            position,
            inv_width,
            inv_height,
            aspect_ratio,
            angle,
        }
    }

    pub fn pixel_to_viewport(&self, x: usize, y: usize) -> cgmath::Vector3<f32> {
        let x = x as f32;
        let y = y as f32;

        let vx =
            (2_f32 * ((x + 0.5_f32) * self.inv_width) - 1_f32) * self.angle * self.aspect_ratio;
        let vy = (1_f32 - 2_f32 * ((y + 0.5_f32) * self.inv_height)) * self.angle;

        let ray_direction = cgmath::vec3(vx, vy, 1.0_f32);
        ray_direction.normalize()
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.position
    }
}
