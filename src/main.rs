mod camera;
mod color;
mod material;
mod math;
mod ray_scanner;
mod scene;
mod sphere;
mod utils;
extern crate cgmath;
extern crate num;

use crate::math::*;
use crate::utils::*;

use std::convert::TryInto;

use image::prelude::*;
use image::{filled_image, BmpEncoder, RgbaPixel};

fn attenuate_color(color: color::Color, attenuation: FloatType) -> color::Color {
    color.attenuate(attenuation)
}

#[allow(dead_code)]
fn random_scene(width: usize, height: usize) -> crate::scene::Scene {
    let mut shapes: Vec<Box<dyn crate::scene::Shape>> = Vec::new();

    shapes.push(Box::new(crate::sphere::Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(material::Lambertian::new(
            vec3(0.5, 0.5, 0.5).try_into().unwrap(),
        )),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_in_range(0.0, 1.0);
            let center = Point3::new(
                (a as FloatType) + 0.9 * random_in_range(0.0, 1.0),
                0.2,
                (b as FloatType) + 0.9 * random_in_range(0.0, 1.0),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                let material: Box<dyn material::Material> = if choose_mat < 0.8 {
                    Box::new(material::Lambertian::new(random_color_in_range(0.0, 1.0)))
                } else if choose_mat < 0.95 {
                    let albedo = random_color_in_range(0.5, 1.0);
                    let fuzz = random_in_range(0.0, 1.0);
                    Box::new(material::Metal::new(albedo, fuzz))
                } else {
                    Box::new(material::Dielectric::new(1.5))
                };

                shapes.push(Box::new(crate::sphere::Sphere::new(center, 0.2, material)));
            }
        }
    }

    shapes.push(Box::new(crate::sphere::Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Box::new(material::Dielectric::new(1.5)),
    )));

    shapes.push(Box::new(crate::sphere::Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Box::new(material::Lambertian::new(
            vec3(0.4, 0.2, 0.1).try_into().unwrap(),
        )),
    )));

    shapes.push(Box::new(crate::sphere::Sphere::new(
        Point3::new(3.0, 1.0, 0.0),
        1.0,
        Box::new(material::Metal::new(
            vec3(0.7, 0.6, 0.5).try_into().unwrap(),
            0.0,
        )),
    )));

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    scene::Scene::new(camera, shapes)
}

#[allow(dead_code)]
fn my_test_scene(width: usize, height: usize) -> crate::scene::Scene {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(-5.0, 2.0, 1.0);
    let lookat = Point3::new(0.0, 0.0, -3.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.1;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(60.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes: Vec<Box<dyn crate::scene::Shape>> = vec![
        Box::new(crate::sphere::Sphere::new(
            Point3::new(-0.5, 0.0, -3.0),
            1.0,
            Box::new(material::Dielectric::new(1.5)),
        )),
        Box::new(crate::sphere::Sphere::new(
            Point3::new(-0.5, 0.0, -3.0),
            -0.999,
            Box::new(material::Dielectric::new(1.5)),
        )),
        Box::new(crate::sphere::Sphere::new(
            Point3::new(0.5, 0.0, -5.0),
            1.0,
            Box::new(material::Metal::new(
                attenuate_color(color::Color::MAGENTA, 0.8),
                0.2,
            )),
        )),
        Box::new(crate::sphere::Sphere::new(
            Point3::new(-0.5, 0.0, -5.0),
            1.0,
            Box::new(material::Metal::new(
                attenuate_color(color::Color::WHITE, 0.8),
                0.0,
            )),
        )),
        Box::new(crate::sphere::Sphere::new(
            Point3::new(0.0, -51.0, -5.0),
            50.0,
            Box::new(material::Lambertian::new(attenuate_color(
                color::Color::YELLOW,
                0.5,
            ))),
        )),
    ];
    scene::Scene::new(camera, shapes)
}

fn main() {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;

    let mut surf = filled_image(WIDTH, HEIGHT, RgbaPixel::BLACK).unwrap();
    let scene = my_test_scene(WIDTH, HEIGHT);
    ray_scanner::scan(&mut surf, &scene);
    BmpEncoder::new()
        .write_image_to_file(&surf, "/Volumes/Unix/src/hello.bmp")
        .unwrap();
}
