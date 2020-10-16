use super::TransformableShape;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

#[derive(Debug, Clone)]
pub struct HitResult<'a> {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub u: FloatType,
    pub v: FloatType,
}

pub trait Shape: Send + Sync + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait SimpleShape: Shape {}

impl Shape for Box<dyn Shape> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.as_ref().intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.as_ref().bounding_box(t0, t1)
    }
}

pub trait CompoundShape: Shape {
    type GeometryIterator: Iterator<Item = Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator;
}

pub trait TypedCompoundShape<S: Shape>: Shape {
    type GeometryIterator: Iterator<Item = S>;

    fn into_typed_geometry_iterator(self) -> Self::GeometryIterator;
}

impl<S: SimpleShape + 'static> CompoundShape for S {
    type GeometryIterator = std::iter::Once<Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        let b: Box<dyn Shape> = Box::new(self);
        std::iter::once(b)
    }
}

impl<S: SimpleShape + 'static> TypedCompoundShape<S> for S {
    type GeometryIterator = std::iter::Once<Self>;

    fn into_typed_geometry_iterator(self) -> Self::GeometryIterator {
        std::iter::once(self)
    }
}

impl CompoundShape for Box<dyn Shape> {
    type GeometryIterator = std::iter::Once<Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        std::iter::once(self)
    }
}

impl TypedCompoundShape<Box<dyn Shape>> for Box<dyn Shape> {
    type GeometryIterator = std::iter::Once<Box<dyn Shape>>;

    fn into_typed_geometry_iterator(self) -> Self::GeometryIterator {
        std::iter::once(self)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct CollectionShape<S: Shape>(Vec<S>);

impl<S: Shape> CollectionShape<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn build() -> Self {
        Self(Vec::new())
    }

    pub fn push<T: TypedCompoundShape<S>>(&mut self, shape: T) {
        self.0.extend(shape.into_typed_geometry_iterator());
    }

    pub fn extend_with_shape<T: TypedCompoundShape<S>>(mut self, shape: T) -> Self {
        self.push(shape);
        self
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, S> {
        self.0.iter()
    }
}

impl<S: Shape> Shape for CollectionShape<S> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.0
            .iter()
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max, stats))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.0
            .iter()
            .map(|a| a.bounding_box(t0, t1).clone())
            .my_fold_first(|a, b| BoundingBox::surrounding_box(&a, &b))
            .unwrap_or(BoundingBox::empty_box())
    }
}

impl<S: TransformableShape> TransformableShape for CollectionShape<S> {
    type Target = CollectionShape<S::Target>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        CollectionShape(
            self.0
                .into_iter()
                .map(|shape| shape.transform(transform))
                .collect(),
        )
    }
}

impl<S: Shape> IntoIterator for CollectionShape<S> {
    type Item = S;
    type IntoIter = <Vec<S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, S: Shape> IntoIterator for &'a CollectionShape<S> {
    type Item = &'a S;
    type IntoIter = std::slice::Iter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<S: Shape> std::iter::FromIterator<S> for CollectionShape<S> {
    fn from_iter<Iter: IntoIterator<Item = S>>(iter: Iter) -> Self {
        let shapes = iter.into_iter().collect();
        Self(shapes)
    }
}

impl<S: Shape, C: TypedCompoundShape<S>> Extend<C> for CollectionShape<S> {
    fn extend<Iter: IntoIterator<Item = C>>(&mut self, iter: Iter) {
        for shape in iter {
            self.push(shape)
        }
    }
}

impl<S: Shape> TypedCompoundShape<S> for CollectionShape<S> {
    type GeometryIterator = std::vec::IntoIter<S>;

    fn into_typed_geometry_iterator(self) -> Self::GeometryIterator {
        self.0.into_iter()
    }
}

pub trait IntoShape {
    type Shape: Shape;

    fn into_shape(self) -> Self::Shape;
}

impl<S: Shape, IntoIter: IntoIterator<Item = S>> IntoShape for IntoIter {
    type Shape = CollectionShape<S>;

    fn into_shape(self) -> Self::Shape {
        CollectionShape(self.into_iter().collect())
    }
}
