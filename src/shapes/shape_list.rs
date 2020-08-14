use super::{GeometryObject, HitResult, Hittable};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::RenderStatsCollector;

#[derive(Debug, Clone)]
pub struct ShapeList {
    shapes: Vec<Box<dyn Hittable>>,
}

impl ShapeList {
    pub fn build() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn push<T: GeometryObject>(&mut self, shape: T) {
        self.shapes.extend(shape.into_geometry_iterator());
    }

    pub fn extend_with_shape<T: GeometryObject>(mut self, shape: T) -> Self {
        self.push(shape);
        self
    }
}

impl Hittable for ShapeList {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
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

impl<T: GeometryObject> std::iter::FromIterator<T> for ShapeList {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let shapes = iter
            .into_iter()
            .flat_map(|shape| shape.into_geometry_iterator())
            .collect();
        Self { shapes }
    }
}

impl<T: GeometryObject> Extend<T> for ShapeList {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        for item in iter {
            self.push(item);
        }
    }
}

impl GeometryObject for ShapeList {
    type GeometryIterator = std::vec::IntoIter<Box<dyn Hittable>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        self.shapes.into_iter()
    }
}

#[macro_export]
macro_rules! shapes {
    () => { $crate::ShapeList::build() };
    ($($x:expr),+ $(,)?) => {
        $crate::ShapeList::build()$(.extend_with_shape($x))+
    };
}
