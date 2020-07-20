mod camera;
mod color;
mod material;
mod ray_scanner;
mod scene;
mod sphere;
extern crate cgmath;
extern crate num;

use std::convert::TryInto;

use image::prelude::*;
use image::{filled_image, BmpEncoder, RgbaPixel};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn attenuate_color(color: color::Color, attenuation: f32) -> color::Color {
    (cgmath::Vector4::from(color).truncate() * attenuation)
        .try_into()
        .unwrap()
}

fn main() {
    let mut surf = filled_image(WIDTH, HEIGHT, RgbaPixel::BLACK).unwrap();
    let camera = camera::Camera::new(
        cgmath::vec3(0.0, 0.0, -20.0),
        surf.width(),
        surf.height(),
        cgmath::Deg(30.0).into(),
    );
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-1.0, -1.0, -2.0),
            1.0,
            Box::new(material::Lambertian::new(attenuate_color(
                color::Color::RED,
                0.5,
            ))),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, 0.0, 0.0),
            1.0,
            Box::new(material::Metal::new(attenuate_color(
                color::Color::GREEN,
                0.5,
            ))),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, 51.0, 0.0),
            50.0,
            Box::new(material::Lambertian::new(attenuate_color(
                color::Color::YELLOW,
                0.5,
            ))),
        )),
    ];
    let scene = scene::Scene::new(camera, shapes);
    ray_scanner::scan(&mut surf, &scene);
    BmpEncoder::new()
        .write_image_to_file(&surf, "/Volumes/Unix/src/hello.bmp")
        .unwrap();
}
