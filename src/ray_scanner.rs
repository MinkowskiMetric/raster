use crate::math::*;
use crate::scene::{PreparedScene, Scene};
use crate::utils::*;
use crate::TracingStats;
use crate::{constants, Color, Hittable, PartialScatterResult, ScatterResult};
use futures::future::join_all;
use std::slice::{Chunks, ChunksMut};

use std::convert::TryInto;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray {
    pub origin: M256Point3,
    pub direction: M256Vector3,
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
            direction: direction.into(),
            inv_direction: inv_direction.into(),
            sign: sign.into(),
            time,
        }
    }
}

pub struct VectorImage {
    width: usize,
    data: Box<[cgmath::Vector4<FloatType>]>,
}

pub struct Pixels<'a> {
    chunks: Chunks<'a, cgmath::Vector4<FloatType>>,
}

impl<'a> Iterator for Pixels<'a> {
    type Item = &'a cgmath::Vector4<FloatType>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|slice| &slice[0])
    }
}

pub struct PixelsMut<'a> {
    chunks: ChunksMut<'a, cgmath::Vector4<FloatType>>,
}

impl<'a> Iterator for PixelsMut<'a> {
    type Item = &'a mut cgmath::Vector4<FloatType>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|slice| &mut slice[0])
    }
}

pub struct EnumeratePixelsMut<'a> {
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

pub async fn scan(
    scene: Scene,
    image_width: usize,
    image_height: usize,
    t0: FloatType,
    t1: FloatType,
    thread_count: usize,
    min_passes: usize,
) -> VectorImage {
    let thread_count = thread_count.max(1);
    let min_passes = min_passes.max(1);
    let passes_per_thread = (min_passes + thread_count - 1) / thread_count;

    let start_time = std::time::Instant::now();

    let scene = Arc::new(PreparedScene::make(scene, t0, t1));

    let futures = (0..thread_count).into_iter().map(|_| {
        let thread_scene = scene.clone();
        tokio::task::spawn_blocking(move || {
            scan_batch(image_width, image_height, passes_per_thread, &thread_scene)
        })
    });

    let (vector_image, tracing_stats) = join_all(futures)
        .await
        .into_iter()
        .my_fold_first(|a, b| match (a, b) {
            (Ok(a), Ok(b)) => Ok((a.0 + b.0, a.1 + b.1)),
            (Err(a), _) => Err(a),
            (_, Err(b)) => Err(b),
        })
        .unwrap()
        .unwrap();

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Total runtime: {} seconds", elapsed);
    println!("Tracing stats: {:#?}", tracing_stats);

    vector_image
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
                let (s, t) = (
                    ((x as FloatType) + random_in_range(-0.5, 0.5)) / image_width,
                    ((image_height - 1.0 - (y as FloatType)) + random_in_range(-0.5, 0.5))
                        / image_height,
                );
                let ray = scene.camera().make_ray(s, t);

                cgmath::Vector4::from(trace(&ray, scene, &mut stats))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
    }

    (image, stats)
}

const MAX_DEPTH: usize = 50;

#[derive(Debug, Clone, Copy)]
struct ScatterStackRecord {
    partial: PartialScatterResult,
    emitted: Color,
}

type FixedSizeAttenuationStack<'a> =
    crate::fixed_size_stack::FixedSizeStack<'a, ScatterStackRecord>;

fn collapse_color_stack<'a>(mut stack: FixedSizeAttenuationStack<'a>, input_color: Color) -> Color {
    let mut color = input_color;

    while let Some(scatter_record) = stack.pop() {
        let calc_color = Vector4::from(scatter_record.emitted)
            + scatter_record
                .partial
                .attenuation
                .extend(1.0)
                .mul_element_wise(Vector4::from(color));

        color = calc_color.try_into().unwrap();
    }

    // We need to ensure that the alpha channel is 1 when we come out of here, because that is used
    // later to average the samples.
    Vector4::from(color)
        .truncate()
        .extend(1.0)
        .try_into()
        .unwrap()
}

pub fn trace(ray: &Ray, scene: &PreparedScene, stats: &mut TracingStats) -> Color {
    let mut attenuation_stack_data = [None; MAX_DEPTH];
    let mut attenuation_stack = FixedSizeAttenuationStack::new(&mut attenuation_stack_data);

    let mut current_ray = ray.clone();

    loop {
        stats.count_ray_cast();

        if attenuation_stack.len() >= MAX_DEPTH {
            // We cannot recurse any further, there is no point doing another hit test
            return collapse_color_stack(attenuation_stack, constants::BLACK);
        } else {
            if let Some(hit_result) =
                scene.intersect(&current_ray, 0.001, constants::INFINITY, stats)
            {
                // We hit an object. First see if it emitted any light
                let emitted =
                    hit_result
                        .material
                        .emitted(hit_result.hit_point, hit_result.u, hit_result.v);
                if let Some(ScatterResult { partial, scattered }) =
                    hit_result.material.scatter(&current_ray, &hit_result)
                {
                    attenuation_stack.push(ScatterStackRecord { partial, emitted });
                    current_ray = scattered;
                } else {
                    return collapse_color_stack(attenuation_stack, emitted);
                }
            } else {
                // We did not intersect with any objects, so sample the sky
                return collapse_color_stack(attenuation_stack, scene.sky().sample(&current_ray));
            }
        }
    }
}
