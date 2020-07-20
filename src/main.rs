mod camera;
mod ray_scanner;
mod scene;
mod sphere;
extern crate cgmath;
extern crate num;

use image::prelude::*;
use image::{filled_image, BmpEncoder, RgbaPixel};

fn main() {
    let mut surf = filled_image(320, 240, RgbaPixel::BLACK).unwrap();
    let camera = camera::Camera::new(
        cgmath::vec3(0.0, 0.0, -20.0),
        320,
        240,
        cgmath::Deg(30.0).into(),
    );
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-1.0, -1.0, 0.0),
            1.0,
            RgbaPixel::RED,
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, 0.0, -5.0),
            1.0,
            RgbaPixel::GREEN,
        )),
    ];
    let scene = scene::Scene::new(camera, shapes);
    ray_scanner::scan(&mut surf, &scene);
    BmpEncoder::new()
        .write_image_to_file(&surf, "/Volumes/Unix/src/hello.bmp")
        .unwrap();
}
