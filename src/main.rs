mod camera;
mod color;
mod material;
mod ray_scanner;
mod scene;
mod sphere;
mod utils;
extern crate cgmath;
extern crate num;
use cgmath::prelude::*;

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
    let aspect_ratio = (WIDTH as f32) / (HEIGHT as f32);

    let lookfrom = cgmath::Point3::new(-5.0, 2.0, 1.0);
    let lookat = cgmath::Point3::new(0.0, 0.0, -3.0);
    let vup = cgmath::vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.1;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        cgmath::Deg(60.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-0.5, 0.0, -3.0),
            1.0,
            Box::new(material::Dielectric::new(1.5)),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-0.5, 0.0, -3.0),
            -0.999,
            Box::new(material::Dielectric::new(1.5)),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.5, 0.0, -5.0),
            1.0,
            Box::new(material::Metal::new(
                attenuate_color(color::Color::MAGENTA, 0.8),
                0.2,
            )),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(-0.5, 0.0, -5.0),
            1.0,
            Box::new(material::Metal::new(
                attenuate_color(color::Color::WHITE, 0.8),
                0.0,
            )),
        )),
        Box::new(crate::sphere::Sphere::new(
            cgmath::vec3(0.0, -51.0, -5.0),
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
