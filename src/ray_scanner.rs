use crate::{
    constants,
    math::*,
    scene::{PreparedScene, Scene},
    utils::*,
    Color, PartialScatterResult, Ray, RenderStatsAccumulator, RenderStatsCollector, ScatterResult,
    TracingStats, VisibleIntersection,
};
use futures::future::join_all;
use std::{
    mem::MaybeUninit,
    slice::{Chunks, ChunksMut},
};

use std::convert::TryInto;
use std::sync::{Arc, RwLock};

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
            chunks: self.data.chunks(1),
        }
    }

    pub fn pixels_mut(&mut self) -> PixelsMut {
        PixelsMut {
            chunks: self.data.chunks_mut(1),
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
            .fold((), |_, (dst, src)| *dst += *src);

        self
    }
}

pub async fn scan<StatsAccumulator: 'static + RenderStatsAccumulator + Sync + Send>(
    scene: Scene,
    (image_width, image_height): (usize, usize),
    t0: FloatType,
    t1: FloatType,
    thread_count: usize,
    min_passes: usize,
    stats: Arc<RwLock<StatsAccumulator>>,
) -> VectorImage {
    let thread_count = thread_count.max(1);
    let min_passes = min_passes.max(1);
    let passes_per_thread = (min_passes + thread_count - 1) / thread_count;

    let scene = Arc::new(PreparedScene::make(scene, t0, t1));

    let futures = (0..thread_count).into_iter().map(|_| {
        let thread_scene = scene.clone();
        let thread_stats = stats.clone();
        tokio::task::spawn_blocking(move || {
            scan_batch(
                image_width,
                image_height,
                passes_per_thread,
                &thread_scene,
                thread_stats.as_ref(),
            )
        })
    });

    let mut thread_results = join_all(futures).await.into_iter();
    let result = thread_results.next();
    result
        .map(|result| {
            thread_results.fold(result, |l, r| match (l, r) {
                (Ok(l), Ok(r)) => Ok(l + r),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            })
        })
        .transpose()
        .unwrap()
        .unwrap()
}

fn scan_batch(
    image_width: usize,
    image_height: usize,
    pass_count: usize,
    scene: &PreparedScene,
    stats: &RwLock<impl RenderStatsAccumulator>,
) -> VectorImage {
    let mut image = VectorImage::new(image_width, image_height);
    let (image_width, image_height) = (image_width as FloatType, image_height as FloatType);
    let mut pixel_stats = TracingStats::new();

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

                let ret = cgmath::Vector4::from(trace(&ray, scene));

                pixel_stats.count_pixel();

                if let Ok(mut lock) = stats.try_write() {
                    let next_stats = std::mem::replace(&mut pixel_stats, TracingStats::new());
                    lock.add_stats(next_stats.into());
                }

                ret
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
    }

    image
}

const MAX_DEPTH: usize = 50;

#[derive(Debug, Clone, Copy)]
struct ScatterStackRecord {
    partial: PartialScatterResult,
    emitted: Color,
}

type FixedSizeAttenuationStack<'a> =
    crate::fixed_size_stack::FixedSizeStack<'a, ScatterStackRecord>;

fn collapse_color_stack(mut stack: FixedSizeAttenuationStack<'_>, input_color: Color) -> Color {
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

pub fn trace(ray: &Ray, scene: &PreparedScene) -> Color {
    let mut attenuation_stack_data: [_; MAX_DEPTH] = MaybeUninit::uninit_array();
    let mut attenuation_stack = FixedSizeAttenuationStack::new(&mut attenuation_stack_data);

    let mut current_ray = *ray;

    loop {
        if let Some(hit_result) = scene.intersect(&current_ray, 0.001, constants::INFINITY) {
            let (hit_result, material) = hit_result.split();
            let (emitted, scatter) = material.base_scatter(&current_ray, hit_result).split();

            if let Some(ScatterResult { partial, scattered }) = scatter {
                if !attenuation_stack.try_push(ScatterStackRecord { partial, emitted }) {
                    // We cannot recurse any further, so stop here and return black
                    return collapse_color_stack(attenuation_stack, constants::BLACK);
                }

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
