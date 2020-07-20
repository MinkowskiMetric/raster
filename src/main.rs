mod camera;
mod color;
mod ray_scanner;
mod scene;
mod sphere;
extern crate cgmath;
extern crate num;

use image::prelude::*;
use image::{filled_image, BmpEncoder, RgbaPixel};

fn main() {
    let mut surf = filled_image(1920, 1080, RgbaPixel::BLACK).unwrap();
    let camera = camera::Camera::new(
        cgmath::vec3(0.0, 0.0, -20.0),
        surf.width(),
        surf.height(),
        cgmath::Deg(30.0).into(),
    );
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-1.0, -1.0, 0.0),
            1.0,
            color::Color::RED,
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, 0.0, -5.0),
            1.0,
            color::Color::GREEN,
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, 51.0, -5.0),
            50.0,
            color::Color::GREEN,
        )),
    ];
    let scene = scene::Scene::new(camera, shapes);
    ray_scanner::scan(&mut surf, &scene);
    BmpEncoder::new()
        .write_image_to_file(&surf, "/Volumes/Unix/src/hello.bmp")
        .unwrap();
}
