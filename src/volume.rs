use crate::aabb::BoundingBox;
use crate::hittable::{HitResult, Hittable, SharedHittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::stats::TracingStats;
use crate::utils::*;

#[derive(Debug, PartialEq, Copy, Clone)]
enum ComparatorAxis {
    X,
    Y,
    Z,
}

fn random_axis() -> ComparatorAxis {
    let random = random_in_range(0.0, 3.0);
    if random < 1.0 {
        ComparatorAxis::X
    } else if random < 2.0 {
        ComparatorAxis::Y
    } else {
        ComparatorAxis::Z
    }
}

fn get_axis_values(
    hittable: &Option<SharedHittable>,
    t0: FloatType,
    t1: FloatType,
    axis: ComparatorAxis,
) -> FloatType {
    if let Some(hittable) = hittable {
        let bounding_box = hittable.bounding_box(t0, t1);

        match axis {
            ComparatorAxis::X => bounding_box.min_point().x(),
            ComparatorAxis::Y => bounding_box.min_point().y(),
            ComparatorAxis::Z => bounding_box.min_point().z(),
        }
    } else {
        0.0
    }
}

fn random_axis_comparator(
    t0: FloatType,
    t1: FloatType,
) -> impl Fn(&Option<SharedHittable>, &Option<SharedHittable>) -> std::cmp::Ordering {
    let comparator_axis = random_axis();

    move |left, right| {
        let (left, right) = (
            get_axis_values(left, t0, t1, comparator_axis),
            get_axis_values(right, t0, t1, comparator_axis),
        );
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Clone, Debug)]
enum InnerVolume {
    NoChild,
    SingleChild(SharedHittable),
    TwoChild {
        left: Box<Volume>,
        right: Box<Volume>,
    },
}

fn compute_inner_volume_bounding_box(
    inner_volume: &InnerVolume,
    t0: FloatType,
    t1: FloatType,
) -> BoundingBox {
    match inner_volume {
        InnerVolume::NoChild => BoundingBox::empty_box(),
        InnerVolume::SingleChild(child) => child.bounding_box(t0, t1),
        InnerVolume::TwoChild { left, right } => {
            BoundingBox::surrounding_box(&left.bounding_box(t0, t1), &right.bounding_box(t0, t1))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Volume {
    inner_volume: InnerVolume,
    bounding_box: BoundingBox,
}

type FixedSizeVolumeStack<'a, 'b> = crate::fixed_size_stack::FixedSizeStack<'a, &'b Volume>;

fn replace_hit_result<'a>(
    hit_result: Option<HitResult<'a>>,
    new_value: Option<HitResult<'a>>,
) -> Option<HitResult<'a>> {
    match (hit_result, new_value) {
        (Some(left), Some(right)) if left.distance < right.distance => Some(left),
        (_, Some(right)) => Some(right),
        (Some(left), _) => Some(left),
        _ => None,
    }
}

impl Volume {
    pub fn from_shapes(
        shapes: impl IntoIterator<Item = SharedHittable>,
        t0: FloatType,
        t1: FloatType,
    ) -> Self {
        let mut shapes: Vec<_> = shapes.into_iter().map(|s| Some(s)).collect();
        Self::from_shapes_slice(&mut shapes, t0, t1)
    }

    fn from_shapes_slice(
        shapes: &mut [Option<SharedHittable>],
        t0: FloatType,
        t1: FloatType,
    ) -> Self {
        if shapes.len() == 0 {
            Self::from_inner_volume(InnerVolume::NoChild, t0, t1)
        } else if shapes.len() == 1 {
            Self::from_inner_volume(InnerVolume::SingleChild(shapes[0].take().unwrap()), t0, t1)
        } else {
            // Sort the shapes list according to a random axis
            shapes.sort_by(random_axis_comparator(t0, t1));

            let pivot = shapes.len() / 2;
            Self::from_inner_volume(
                InnerVolume::TwoChild {
                    left: Box::new(Self::from_shapes_slice(&mut shapes[0..pivot], t0, t1)),
                    right: Box::new(Self::from_shapes_slice(&mut shapes[pivot..], t0, t1)),
                },
                t0,
                t1,
            )
        }
    }

    fn from_inner_volume(inner_volume: InnerVolume, t0: FloatType, t1: FloatType) -> Self {
        let bounding_box = compute_inner_volume_bounding_box(&inner_volume, t0, t1);

        Volume {
            inner_volume,
            bounding_box,
        }
    }

    #[target_feature(enable = "avx")]
    unsafe fn intersect_avx<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let mut volume_stack_data = [None; 50];
        let mut volume_stack = FixedSizeVolumeStack::new(&mut volume_stack_data);
        let mut hit_result = None;

        volume_stack.push(self);

        while let Some(v) = volume_stack.pop() {
            stats.count_bounding_box_test();
            if v.bounding_box.intersect_avx(ray, t_min, t_max) {
                if let InnerVolume::TwoChild { left, right } = &v.inner_volume {
                    // We know that we can always push one value onto the stack here
                    // because we just popped a value off
                    volume_stack.push(right.as_ref());

                    // But this push might panic if the stack is full so check for that and
                    // recurse if necessary
                    if volume_stack.is_full() {
                        hit_result = replace_hit_result(
                            hit_result,
                            left.intersect_avx(ray, t_min, t_max, stats),
                        );
                    } else {
                        volume_stack.push(left.as_ref());
                    }
                } else if let InnerVolume::SingleChild(c) = &v.inner_volume {
                    hit_result =
                        replace_hit_result(hit_result, c.intersect(ray, t_min, t_max, stats));
                }
            }
        }

        hit_result
    }
}

impl Hittable for Volume {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        unsafe { self.intersect_avx(ray, t_min, t_max, stats) }
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        self.bounding_box.clone()
    }
}
