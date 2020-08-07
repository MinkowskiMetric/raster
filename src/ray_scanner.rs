use crate::color::Color;
use crate::hittable::Hittable;
use crate::material::ScatterResult;
use crate::math::*;
use crate::scene::{PreparedScene, Scene};
use crate::stats::TracingStats;
use crate::utils::*;
use std::slice::{Chunks, ChunksMut};
use std::thread;

use image::{Pixel, SurfaceMut};

use std::convert::TryInto;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
    pub origin: M256Point3,
    pub direction: Vector3,
    pub inv_direction: M256Vector3,
    pub sign: M256Vector3,
    pub time: FloatType,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3, time: FloatType) -> Self {
        let inv_direction = 1.0 / direction;
        let sign = Vector4::new(
            if inv_direction.x < 0.0 { -1.0 } else { 0.0 },
            if inv_direction.y < 0.0 { -1.0 } else { 0.0 },
            if inv_direction.z < 0.0 { -1.0 } else { 0.0 },
            0.0,
        );

        Self {
            origin: origin.into(),
            direction,
            inv_direction: inv_direction.into(),
            sign: sign.into(),
            time,
        }
    }
}

struct VectorImage {
    width: usize,
    data: Box<[cgmath::Vector4<FloatType>]>,
}

struct Pixels<'a> {
    chunks: Chunks<'a, cgmath::Vector4<FloatType>>,
}

impl<'a> Iterator for Pixels<'a> {
    type Item = &'a cgmath::Vector4<FloatType>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|slice| &slice[0])
    }
}

struct PixelsMut<'a> {
    chunks: ChunksMut<'a, cgmath::Vector4<FloatType>>,
}

impl<'a> Iterator for PixelsMut<'a> {
    type Item = &'a mut cgmath::Vector4<FloatType>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|slice| &mut slice[0])
    }
}

struct EnumeratePixelsMut<'a> {
    pixels: PixelsMut<'a>,
    x: usize,
    y: usize,
    width: usize,
}

impl<'a> Iterator for EnumeratePixelsMut<'a> {
    type Item = (usize, usize, &'a mut cgmath::Vector4<FloatType>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        self.pixels.next().map(|p| (x, y, p))
    }
}

impl VectorImage {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![cgmath::vec4(0.0, 0.0, 0.0, 0.0); width * height];
        Self {
            width,
            data: data.into_boxed_slice(),
        }
    }

    pub fn pixels(&self) -> Pixels {
        Pixels {
            chunks: self.data.chunks(1 as usize),
        }
    }

    pub fn pixels_mut(&mut self) -> PixelsMut {
        PixelsMut {
            chunks: self.data.chunks_mut(1 as usize),
        }
    }

    pub fn enumerate_pixels_mut(&mut self) -> EnumeratePixelsMut {
        let width = self.width;
        EnumeratePixelsMut {
            pixels: self.pixels_mut(),
            x: 0,
            y: 0,
            width,
        }
    }
}

impl std::ops::Add for VectorImage {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.pixels_mut()
            .zip(other.pixels())
            .fold({}, |_, (dst, src)| *dst = *dst + *src);

        self
    }
}

pub fn scan<P: Pixel + From<Color>>(
    image: &mut impl SurfaceMut<P>,
    scene: Scene,
    t0: FloatType,
    t1: FloatType,
    thread_count: usize,
    min_passes: usize,
) {
    let passes_per_thread = (min_passes + thread_count - 1) / thread_count;

    let start_time = std::time::Instant::now();

    let (image_width, image_height) = image.dimensions();

    let scene = PreparedScene::make(scene, t0, t1);

    let (vector_image, tracing_stats) = (0..thread_count)
        .into_iter()
        .map(|_a| {
            let thread_scene = scene.clone();
            thread::spawn(move || {
                scan_batch(image_width, image_height, passes_per_thread, &thread_scene)
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|jh| jh.join())
        .my_fold_first(|a, b| {
            if let Ok(a) = a {
                if let Ok(b) = b {
                    Ok((a.0 + b.0, a.1 + b.1))
                } else {
                    b
                }
            } else {
                a
            }
        })
        .unwrap()
        .unwrap();

    vector_image
        .pixels()
        .zip(image.pixels_mut())
        .fold({}, |_, (src, dst)| {
            let color = src / src.w;
            let color: Color = color.try_into().unwrap();
            *dst = color.gamma(2.0).into();
        });

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Total runtime: {} seconds", elapsed);
    println!("Tracing stats: {:#?}", tracing_stats);
}

fn scan_batch(
    image_width: usize,
    image_height: usize,
    pass_count: usize,
    scene: &PreparedScene,
) -> (VectorImage, TracingStats) {
    let mut stats = TracingStats::new();
    let mut image = VectorImage::new(image_width, image_height);
    let (image_width, image_height) = (image_width as FloatType, image_height as FloatType);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = (0..pass_count)
            .into_iter()
            .map(|_s| {
                let (u, v) = (
                    ((x as FloatType) + random_in_range(-0.5, 0.5)) / image_width,
                    ((y as FloatType) + random_in_range(-0.5, 0.5)) / image_height,
                );
                let ray = scene.camera().make_ray(u, v);

                cgmath::Vector4::from(trace(&ray, scene, &mut stats))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
    }

    (image, stats)
}

const MAX_DEPTH: usize = 50;

type FixedSizeAttenuationStack<'a> = crate::fixed_size_stack::FixedSizeStack<'a, ScatterResult>;

fn single_trace(
    ray: &Ray,
    scene: &PreparedScene,
    stats: &mut TracingStats,
) -> Option<ScatterResult> {
    stats.count_ray_cast();
    scene
        .intersect(ray, 0.001, constants::INFINITY, stats)
        .and_then(|hit_result| hit_result.material.scatter(&ray, &hit_result))
}

pub fn trace(ray: &Ray, scene: &PreparedScene, stats: &mut TracingStats) -> Color {
    let mut attenuation_stack_data = [None; MAX_DEPTH];
    let mut attenuation_stack = FixedSizeAttenuationStack::new(&mut attenuation_stack_data);
    attenuation_stack.push(ScatterResult {
        attenuation: vec3(1.0, 1.0, 1.0).try_into().unwrap(),
        scattered: *ray,
    });

    loop {
        let current_ray = &attenuation_stack.last().unwrap().scattered;

        let scatter_result = single_trace(current_ray, scene, stats);
        if scatter_result.is_some() && attenuation_stack.len() < MAX_DEPTH {
            attenuation_stack.push(scatter_result.unwrap());
        } else {
            // At this point, we've either not hit anything, or we've run out of space in the stack
            let mut color = if scatter_result.is_some() {
                Color::BLACK
            } else {
                let unit_direction = current_ray.direction;
                let t = 0.5 * (1.0 - unit_direction.y);
                (((1.0 - t) * vec3(1.0, 1.0, 1.0)) + (t * vec3(0.5, 0.7, 1.0)))
                    .try_into()
                    .unwrap()
            };

            while let Some(scatter_result) = attenuation_stack.pop() {
                color = scatter_result
                    .attenuation
                    .mul_element_wise(cgmath::Vector4::from(color).truncate())
                    .try_into()
                    .unwrap()
            }

            return color;
        }
    }
}
