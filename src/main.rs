mod aabb;
mod camera;
mod color;
mod fixed_size_stack;
mod hittable;
mod material;
mod math;
mod perlin;
mod ray_scanner;
mod scene;
mod shape_list;
mod sphere;
mod stats;
mod texture;
mod utils;
mod volume;
extern crate cgmath;
extern crate num;

extern crate clap;
use clap::{App, Arg};

use crate::color::Color;
use crate::math::*;
use crate::utils::*;

use crate::hittable::{shapes::*, SharedHittable};
use crate::material::materials::*;
use crate::texture::textures::*;

use std::convert::TryInto;

use image::RgbImage;

fn attenuate_color(color: color::Color, attenuation: FloatType) -> color::Color {
    color.attenuate(attenuation)
}

fn random_scene(width: usize, height: usize) -> (camera::Camera, Vec<SharedHittable>) {
    let mut shapes: Vec<SharedHittable> = Vec::new();

    shapes.push(sphere(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        lambertian(checker_texture(
            solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
            solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
        )),
    ));

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
                    let material = lambertian(solid_texture(random_color_in_range(0.0, 1.0)));
                    moving_sphere((center, 0.0), (center2, 1.0), 0.2, material)
                } else if choose_mat < 0.95 {
                    let albedo = random_color_in_range(0.5, 1.0);
                    let fuzz = random_in_range(0.0, 1.0);
                    let material = metal(albedo, fuzz);
                    sphere(center, 0.2, material)
                } else {
                    let material = dielectric(1.5);
                    sphere(center, 0.2, material)
                });
            }
        }
    }

    shapes.push(sphere(Point3::new(0.0, 1.0, 0.0), 1.0, dielectric(1.5)));

    shapes.push(sphere(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        lambertian(solid_texture(vec3(0.4, 0.2, 0.1).try_into().unwrap())),
    ));

    shapes.push(sphere(
        Point3::new(3.0, 1.0, 0.0),
        1.0,
        metal(vec3(0.7, 0.6, 0.5).try_into().unwrap(), 0.0),
    ));

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

fn my_test_scene(width: usize, height: usize) -> (camera::Camera, Vec<SharedHittable>) {
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
    let shapes: Vec<SharedHittable> = vec![
        sphere(Point3::new(-0.5, 0.0, -3.0), 1.0, dielectric(1.5)),
        sphere(Point3::new(-0.5, 0.0, -3.0), -0.999, dielectric(1.5)),
        sphere(
            Point3::new(0.5, 0.0, -5.0),
            1.0,
            metal(attenuate_color(color::Color::MAGENTA, 0.8), 0.2),
        ),
        sphere(
            Point3::new(-0.5, 0.0, -5.0),
            1.0,
            metal(attenuate_color(color::Color::WHITE, 0.8), 0.0),
        ),
        sphere(
            Point3::new(0.0, -51.0, -5.0),
            50.0,
            lambertian(solid_texture(attenuate_color(color::Color::YELLOW, 0.5))),
        ),
    ];
    (camera, shapes)
}

fn two_spheres(width: usize, height: usize) -> (camera::Camera, Vec<SharedHittable>) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes: Vec<SharedHittable> = vec![
        sphere(
            Point3::new(0.0, -10.0, 0.0),
            10.0,
            lambertian(checker_texture(
                solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
                solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
            )),
        ),
        sphere(
            Point3::new(0.0, 10.0, 0.0),
            10.0,
            lambertian(checker_texture(
                solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
                solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
            )),
        ),
    ];
    (camera, shapes)
}

fn two_perlin_spheres(width: usize, height: usize) -> (camera::Camera, Vec<SharedHittable>) {
    let pertext = noise_texture(4.0);

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes: Vec<SharedHittable> = vec![
        sphere(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            lambertian(pertext.clone()),
        ),
        sphere(Point3::new(0.0, 2.0, 0.0), 2.0, lambertian(pertext.clone())),
    ];

    (camera, shapes)
}

fn textured_earth(width: usize, height: usize) -> (camera::Camera, Vec<SharedHittable>) {
    let earth_bytes = include_bytes!("earthmap.jpg");
    let earth_image = image::load_from_memory(earth_bytes).unwrap();
    let earth_image = image_texture(earth_image);

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = camera::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes: Vec<SharedHittable> = vec![sphere(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        lambertian(earth_image),
    )];

    (camera, shapes)
}

const DEFAULT_WIDTH: usize = 1920;
const DEFAULT_HEIGHT: usize = 1080;
const DEFAULT_MIN_PASSES: usize = 100;
const DEFAULT_THREADS: usize = 8;
const DEFAULT_ENABLE_SPATIAL_PARTITIONING: bool = true;

const BUILTIN_SCENES: [(
    &'static str,
    fn(usize, usize) -> (camera::Camera, Vec<SharedHittable>),
); 5] = [
    ("random", random_scene),
    ("mine", my_test_scene),
    ("twospheres", two_spheres),
    ("twoperlinspheres", two_perlin_spheres),
    ("earth", textured_earth),
];

fn command_line() -> clap::ArgMatches<'static> {
    App::new("raster")
        .version("1.0")
        .author("Stewart Tootill <stewart.tootill@live.co.uk>")
        .about("My raytracer")
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help(&format!("Width of image, defaults to {}", DEFAULT_WIDTH))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .help(&format!("Height of image, defaults to {}", DEFAULT_HEIGHT))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("scene")
                .long("scene")
                .takes_value(true)
                .help(&format!(
                    "Choose a scene to render, defaults to {}",
                    BUILTIN_SCENES[0].0
                ))
                .possible_values(&BUILTIN_SCENES.iter().map(|a| a.0).collect::<Vec<_>>()),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .help(&format!(
                    "Number of threads, defaults to {}",
                    DEFAULT_THREADS
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min-passes")
                .long("min-passes")
                .help(&format!(
                    "Minimum number of passes, defaults to {}",
                    DEFAULT_MIN_PASSES
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("enable-spatial-partitioning")
                .long("enable-spatial-partitioning")
                .possible_values(&["true", "false"])
                .help(&format!(
                    "Enable spatial partitioning, defaults to {}",
                    DEFAULT_ENABLE_SPATIAL_PARTITIONING
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help("File to write to")
                .required(true)
                .index(1)
                .takes_value(true),
        )
        .get_matches()
}

#[tokio::main]
async fn main() {
    let matches = command_line();

    let width = matches
        .value_of("width")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = matches
        .value_of("height")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    let output_file = matches.value_of("output").unwrap().to_string();
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

    let scene_name = matches.value_of("scene").unwrap_or(BUILTIN_SCENES[0].0);
    let (scene_name, scene_function) = BUILTIN_SCENES.iter().find(|a| a.0 == scene_name).unwrap();

    let (camera, shapes) = scene_function(width, height);
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

    let vector_image = ray_scanner::scan(scene, width, height, t0, t1, threads, min_passes).await;

    let mut surf = RgbImage::new(width as u32, height as u32);

    vector_image
        .pixels()
        .zip(surf.pixels_mut())
        .fold({}, |_, (src, dst)| {
            let color = src / src.w;
            let color: Color = color.try_into().unwrap();
            *dst = color.gamma(2.0).into();
        });

    if let Err(e) = surf.save(output_file) {
        println!("Failed to write output: {}", e);
    }
}
