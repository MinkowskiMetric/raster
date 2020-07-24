use crate::color::Color;
use crate::hittable::Hittable;
use crate::material::ScatterResult;
use crate::math::*;
use crate::scene::Scene;
use crate::utils::*;
use std::slice::{Chunks, ChunksMut};
use std::thread;

use image::{Pixel, SurfaceMut};

use std::convert::TryInto;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
    pub inv_direction: Vector3,
    pub sign: [usize; 3],
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3) -> Self {
        let inv_direction = 1.0 / direction;
        let sign = [
            if inv_direction.x < 0.0 { 1 } else { 0 },
            if inv_direction.y < 0.0 { 1 } else { 0 },
            if inv_direction.z < 0.0 { 1 } else { 0 },
        ];

        Self {
            origin,
            direction,
            inv_direction,
            sign,
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
    thread_count: usize,
    min_passes: usize,
) {
    let passes_per_thread = (min_passes + thread_count - 1) / thread_count;

    let start_time = std::time::Instant::now();

    let (image_width, image_height) = image.dimensions();

    let vector_image = (0..thread_count)
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
                    Ok(a + b)
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
}

fn scan_batch(
    image_width: usize,
    image_height: usize,
    pass_count: usize,
    scene: &Scene,
) -> VectorImage {
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

                cgmath::Vector4::from(trace(&ray, scene))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
    }

    image
}

const MAX_DEPTH: usize = 50;

fn assign_swap<T>(target: &mut T, value: T) -> T {
    let mut swap_value = value;
    std::mem::swap(target, &mut swap_value);
    swap_value
}

struct FixedSizeAttenuationStack<'a> {
    data: &'a mut [Option<ScatterResult>],
    top: usize,
}

impl<'a> FixedSizeAttenuationStack<'a> {
    pub fn new(data: &'a mut [Option<ScatterResult>]) -> Self {
        FixedSizeAttenuationStack { data, top: 0 }
    }

    pub fn push(&mut self, scatter_result: ScatterResult) {
        debug_assert!(self.top < self.data.len());
        debug_assert!(self.data[self.top].is_none());

        self.data[self.top] = Some(scatter_result);
        self.top += 1;
    }

    pub fn pop(&mut self) -> Option<ScatterResult> {
        if self.top > 0 {
            self.top -= 1;
            debug_assert!(self.data[self.top].is_some());
            assign_swap(&mut self.data[self.top], None)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&ScatterResult> {
        if self.top > 0 {
            debug_assert!(self.data[self.top - 1].is_some());
            self.data[self.top - 1].as_ref()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.top
    }
}

fn single_trace(ray: &Ray, scene: &Scene) -> Option<ScatterResult> {
    scene
        .intersect(ray, 0.001, constants::INFINITY)
        .and_then(|hit_result| hit_result.material.scatter(&ray, &hit_result))
}

pub fn trace(ray: &Ray, scene: &Scene) -> Color {
    let mut attenuation_stack_data = [None; MAX_DEPTH];
    let mut attenuation_stack = FixedSizeAttenuationStack::new(&mut attenuation_stack_data);
    attenuation_stack.push(ScatterResult {
        attenuation: vec3(1.0, 1.0, 1.0).try_into().unwrap(),
        scattered: *ray,
    });

    loop {
        let current_ray = &attenuation_stack.last().unwrap().scattered;

        let scatter_result = single_trace(current_ray, scene);
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
