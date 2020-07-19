mod camera;
mod encoder;
#[macro_use]
mod pixel;
mod ray_scanner;
mod scene;
mod sphere;
mod surface;
mod writer;
extern crate num;
extern crate cgmath;

use crate::writer::ImageWriter;

fn main() {
    let mut surf = surface::filled_image(320, 240, pixel::RgbaPixel::BLACK).unwrap();
    let camera = camera::Camera::new(cgmath::vec3(0.0,0.0,-20.0), 320, 240, cgmath::Deg(30.0).into());
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(cgmath::vec3(-1.0, -1.0, 0.0), 1.0, crate::pixel::RgbaPixel::RED)),
        Box::new(crate::sphere::Sphere::new(cgmath::vec3(0.0, 0.0, -5.0), 1.0, crate::pixel::RgbaPixel::GREEN)),
    ];
    let scene = scene::Scene::new(camera, shapes);
    ray_scanner::scan(&mut surf, &scene);
    encoder::BmpEncoder::new().write_image_to_file(&surf, "/Volumes/Unix/src/hello.bmp").unwrap();
}
