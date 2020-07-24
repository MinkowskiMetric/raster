mod aabb;
mod camera;
mod color;
mod hittable;
mod material;
mod math;
mod ray_scanner;
mod scene;
mod shape_list;
mod sphere;
mod utils;
mod volume;
extern crate cgmath;
extern crate num;

#[macro_use]
extern crate clap;
use clap::App;

use crate::math::*;
use crate::utils::*;

use std::convert::TryInto;

use image::prelude::*;
use image::{filled_image, BmpEncoder, RgbaPixel};

fn attenuate_color(color: color::Color, attenuation: FloatType) -> color::Color {
    color.attenuate(attenuation)
}

fn random_scene(width: usize, height: usize) -> (camera::Camera, Vec<Box<dyn hittable::Hittable>>) {
    let mut shapes: Vec<Box<dyn hittable::Hittable>> = Vec::new();

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
                shapes.push(if choose_mat < 0.8 {
                    let center2 = center + vec3(0.0, random_in_range(0.0, 0.5), 0.0);
                    let material = Box::new(material::Lambertian::new(random_color_in_range(0.0, 1.0)));
                    Box::new(sphere::MovingSphere::new((center, 0.0), (center2, 1.0), 0.2, material))
                } else if choose_mat < 0.95 {
                    let albedo = random_color_in_range(0.5, 1.0);
                    let fuzz = random_in_range(0.0, 1.0);
                    let material = Box::new(material::Metal::new(albedo, fuzz));
                    Box::new(crate::sphere::Sphere::new(center, 0.2, material))
                } else {
                    let material = Box::new(material::Dielectric::new(1.5));
                    Box::new(crate::sphere::Sphere::new(center, 0.2, material))
                });
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

    (camera, shapes)
}

fn my_test_scene(
    width: usize,
    height: usize,
) -> (camera::Camera, Vec<Box<dyn hittable::Hittable>>) {
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
    let shapes: Vec<Box<dyn hittable::Hittable>> = vec![
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
    (camera, shapes)
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    const DEFAULT_WIDTH: usize = 1920;
    const DEFAULT_HEIGHT: usize = 1080;
    const DEFAULT_MIN_PASSES: usize = 100;
    const DEFAULT_THREADS: usize = 8;
    const DEFAULT_ENABLE_SPATIAL_PARTITIONING: bool = true;

    let width = matches
        .value_of("width")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = matches
        .value_of("height")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    let output_file = matches.value_of("output").unwrap();
    let min_passes = matches
        .value_of("min-passes")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MIN_PASSES);
    let threads = matches
        .value_of("threads")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_THREADS);
    let enable_spatial_partitioning = matches
        .value_of("enable-spatial-partitioning")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(DEFAULT_ENABLE_SPATIAL_PARTITIONING);

    let ((camera, shapes), scene_name) = match matches.value_of("scene") {
        Some("random") => (random_scene(width, height), "random"),
        _ => (my_test_scene(width, height), "mine"),
    };
    let scene = scene::Scene::new(camera, enable_spatial_partitioning, shapes);

    let (t0, t1) = (0.0, 1.0);

    println!(
        "Rendering scene \"{}\" at ({}, {})",
        scene_name, width, height
    );
    println!(
        "Using {} threads, with a minimum of {} passes per pixel",
        threads, min_passes
    );

    let mut surf = filled_image(width, height, RgbaPixel::BLACK).unwrap();
    ray_scanner::scan(&mut surf, scene, t0, t1, threads, min_passes);
    if let Err(e) = BmpEncoder::new().write_image_to_file(&surf, output_file) {
        println!("Failed to write output: {}", e);
    }
}
