use super::{HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::TracingStats;

#[derive(Debug, Clone)]
pub struct ShapeList {
    shapes: Vec<Box<dyn Hittable>>,
}

impl ShapeList {
    pub fn from_shapes(shapes: impl IntoIterator<Item = Box<dyn Hittable>>) -> Self {
        let shapes = shapes.into_iter().collect::<Vec<_>>();

        Self { shapes }
    }
}

impl Hittable for ShapeList {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        self.shapes
            .iter()
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max, stats))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.shapes
            .iter()
            .map(|a| a.bounding_box(t0, t1).clone())
            .my_fold_first(|a, b| BoundingBox::surrounding_box(&a, &b))
            .unwrap_or(BoundingBox::empty_box())
    }
}

impl IntoIterator for ShapeList {
    type Item = Box<dyn Hittable>;
    type IntoIter = std::vec::IntoIter<Box<dyn Hittable>>;

    fn into_iter(self) -> Self::IntoIter {
        self.shapes.into_iter()
    }
}

pub mod factories {
    use super::*;

    pub fn shape_list(shapes: impl IntoIterator<Item = Box<dyn Hittable>>) -> ShapeList {
        ShapeList::from_shapes(shapes)
    }
}

pub mod macro_pieces {
    use super::*;

    pub trait MakeBoxedHittable {
        fn make_boxed_hittable(self) -> Box<dyn Hittable>;
    }

    impl<T: 'static + Hittable> MakeBoxedHittable for T {
        fn make_boxed_hittable(self) -> Box<dyn Hittable> {
            Box::new(self)
        }
    }

    pub fn make_boxed_hittable<T: 'static + MakeBoxedHittable>(t: T) -> Box<dyn Hittable> {
        t.make_boxed_hittable()
    }
}

#[macro_export]
macro_rules! shapes {
    () => { $crate::ShapeList::from_shapes(vec![]) };
    //($elem:expr; $n:expr) => { ... };
    ($($x:expr),+ $(,)?) => { $crate::ShapeList::from_shapes(vec![$($crate::macro_pieces::make_boxed_hittable($x)),+]) };
}
