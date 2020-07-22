use crate::color::Color;
use crate::material::ScatterResult;
use crate::math::*;
use crate::scene::Scene;
use crate::utils::*;

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

const SAMPLE_COUNT: usize = 100;

pub fn scan<P: Pixel + From<Color>>(image: &mut impl SurfaceMut<P>, scene: &Scene) {
    let (image_width, image_height) = image.dimensions();
    let (image_width, image_height) = (image_width as FloatType, image_height as FloatType);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        println!("({}, {})", x, y);
        let colv = (0..SAMPLE_COUNT)
            .into_iter()
            .map(|_s| {
                (
                    (x as FloatType) + random_in_range(-0.5, 0.5),
                    (y as FloatType) - random_in_range(-0.5, 0.5),
                )
            })
            .map(|(x, y)| {
                let (u, v) = (x / image_width, y / image_height);
                let ray = scene.camera().make_ray(u, v);

                cgmath::Vector4::from(trace(&ray, scene))
            })
            .fold(cgmath::vec4(0.0, 0.0, 0.0, 0.0), |sum, v| sum + v);
        let colv = colv / (SAMPLE_COUNT as FloatType);
        let col: Color = colv.try_into().unwrap();
        *pixel = col.gamma(2.0).into();
    }
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

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn len(&self) -> usize {
        self.top
    }
}

fn single_trace(ray: &Ray, scene: &Scene) -> Option<ScatterResult> {
    scene
        .get_shapes(&ray)
        .filter_map(|shape| {
            shape
                .intersect(&ray, 0.001, constants::INFINITY)
                .map(|distance| (shape, distance))
        })
        .min_by(|(_, xr), (_, yr)| {
            xr.distance
                .partial_cmp(&yr.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|(_, hit_record)| hit_record.material.scatter(&ray, &hit_record))
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
