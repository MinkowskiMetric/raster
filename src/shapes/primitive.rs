use super::{CompoundShape, HitResult, Shape, TransformablePrimitive, TransformableShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

#[derive(Debug, Clone)]
pub struct PrimitiveHitResult {
    distance: FloatType,
    hit_point: Point3,
    surface_normal: Vector3,
    tangent: Vector3,
    bitangent: Vector3,
    front_face: bool,
    uv: (FloatType, FloatType),
}

impl PrimitiveHitResult {
    pub fn new(
        distance: FloatType,
        hit_point: Point3,
        surface_normal: Vector3,
        tangent: Vector3,
        bitangent: Vector3,
        front_face: bool,
        uv: (FloatType, FloatType),
    ) -> Self {
        Self {
            distance,
            hit_point,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            uv,
        }
    }

    pub fn distance(&self) -> FloatType {
        self.distance
    }

    pub fn set_distance(&mut self, distance: FloatType) {
        self.distance = distance
    }

    pub fn hit_point(&self) -> Point3 {
        self.hit_point
    }

    pub fn set_hit_point(&mut self, hit_point: Point3) {
        self.hit_point = hit_point
    }

    pub fn surface_normal(&self) -> Vector3 {
        self.surface_normal
    }

    pub fn set_surface_normal(&mut self, surface_normal: Vector3) {
        self.surface_normal = surface_normal
    }

    pub fn tangent(&self) -> Vector3 {
        self.tangent
    }

    pub fn set_tangent(&mut self, tangent: Vector3) {
        self.tangent = tangent
    }

    pub fn bitangent(&self) -> Vector3 {
        self.bitangent
    }

    pub fn set_bitangent(&mut self, bitangent: Vector3) {
        self.bitangent = bitangent
    }

    pub fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn set_front_face(&mut self, front_face: bool) {
        self.front_face = front_face
    }

    pub fn uv(&self) -> (FloatType, FloatType) {
        self.uv
    }
}

pub trait Primitive: Send + Sync + std::fmt::Debug {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait UntransformedPrimitive: Primitive {}

pub trait IntoAggregatePrimitive {
    type Aggregate: Primitive;

    fn into_aggregate_primitive(self) -> Self::Aggregate;
}

#[derive(Clone, Debug)]
pub struct PrimitiveAggregator<
    'a,
    P: 'a + Primitive,
    Iter: Iterator<Item = &'a P> + Clone + Send + Sync + std::fmt::Debug,
> {
    iter: Iter,
}

impl<
        'a,
        P: 'a + Primitive,
        Iter: Iterator<Item = &'a P> + Clone + Send + Sync + std::fmt::Debug,
    > Primitive for PrimitiveAggregator<'a, P, Iter>
{
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        self.iter
            .clone()
            .filter_map(|shape| shape.intersect(ray, t_min, t_max, stats))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.iter
            .clone()
            .map(|a| a.bounding_box(t0, t1))
            .my_fold_first(|a, b| BoundingBox::surrounding_box(&a, &b))
            .unwrap_or_else(BoundingBox::empty_box)
    }
}

impl<
        'a,
        P: 'a + Primitive,
        Iter: Iterator<Item = &'a P> + Clone + Send + Sync + std::fmt::Debug,
    > IntoAggregatePrimitive for Iter
{
    type Aggregate = PrimitiveAggregator<'a, P, Iter>;

    fn into_aggregate_primitive(self) -> Self::Aggregate {
        Self::Aggregate { iter: self }
    }
}

#[derive(Debug, Clone)]
pub struct SkinnedPrimitive<P: Primitive, M: Material> {
    primitives: Vec<P>,
    material: M,
}

impl<P: Primitive, M: Material> SkinnedPrimitive<P, M> {
    pub fn new<Iter: IntoIterator<Item = P>>(material: M, iter: Iter) -> Self {
        Self {
            material,
            primitives: iter.into_iter().collect(),
        }
    }
}

impl<P: Primitive, M: Material> Shape for SkinnedPrimitive<P, M> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.primitives
            .iter()
            .into_aggregate_primitive()
            .intersect(ray, t_min, t_max, stats)
            .map(|hit_result| HitResult::new(hit_result, &self.material))
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.primitives
            .iter()
            .into_aggregate_primitive()
            .bounding_box(t0, t1)
    }
}

impl<P: TransformablePrimitive, M: Material> TransformableShape for SkinnedPrimitive<P, M> {
    type Target = SkinnedPrimitive<P::Target, M>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            primitives: self
                .primitives
                .into_iter()
                .map(|primitive| primitive.transform(transform))
                .collect(),
            material: self.material,
        }
    }
}

pub struct BoxedSkinnedPrimitiveIterator<
    P: Primitive,
    M: Material + Clone,
    Iter: Iterator<Item = P>,
> {
    material: M,
    iter: Iter,
}

impl<
        P: 'static + SkinnablePrimitive<M>,
        M: 'static + Material + Clone,
        Iter: Iterator<Item = P>,
    > Iterator for BoxedSkinnedPrimitiveIterator<P, M, Iter>
{
    type Item = Box<dyn Shape>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|primitive| {
            let b: Box<dyn Shape> = Box::new(primitive.apply_material(self.material.clone()));
            b
        })
    }
}

impl<P: 'static + SkinnablePrimitive<M>, M: 'static + Material + Clone> IntoIterator
    for SkinnedPrimitive<P, M>
{
    type Item = Box<dyn Shape>;
    type IntoIter = BoxedSkinnedPrimitiveIterator<P, M, <Vec<P> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        BoxedSkinnedPrimitiveIterator {
            material: self.material,
            iter: self.primitives.into_iter(),
        }
    }
}

impl<P: 'static + SkinnablePrimitive<M>, M: 'static + Material + Clone> CompoundShape
    for SkinnedPrimitive<P, M>
{
    type GeometryIterator = BoxedSkinnedPrimitiveIterator<P, M, <Vec<P> as IntoIterator>::IntoIter>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        BoxedSkinnedPrimitiveIterator {
            material: self.material,
            iter: self.primitives.into_iter(),
        }
    }
}

pub struct SkinnedPrimitiveIterator<P: Primitive, M: Material + Clone, Iter: Iterator<Item = P>> {
    material: M,
    iter: Iter,
}

impl<
        P: 'static + SkinnablePrimitive<M>,
        M: 'static + Material + Clone,
        Iter: Iterator<Item = P>,
    > Iterator for SkinnedPrimitiveIterator<P, M, Iter>
{
    type Item = P::Target;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|primitive| primitive.apply_material(self.material.clone()))
    }
}

pub trait SkinnablePrimitive<M: Material>: Primitive {
    type Target: Shape;

    fn apply_material(self, material: M) -> Self::Target;
}

impl<M: Material, P: UntransformedPrimitive> SkinnablePrimitive<M> for P {
    type Target = SkinnedPrimitive<P, M>;

    fn apply_material(self, material: M) -> Self::Target {
        SkinnedPrimitive {
            material,
            primitives: vec![self],
        }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct CompoundPrimitive<P: Primitive>(Vec<P>);

impl<P: Primitive> Primitive for CompoundPrimitive<P> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        self.0
            .iter()
            .into_aggregate_primitive()
            .intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.0
            .iter()
            .into_aggregate_primitive()
            .bounding_box(t0, t1)
    }
}

impl<P: TransformablePrimitive> TransformablePrimitive for CompoundPrimitive<P> {
    type Target = CompoundPrimitive<P::Target>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        CompoundPrimitive(
            self.0
                .into_iter()
                .map(|primitive| primitive.transform(transform))
                .collect(),
        )
    }
}

impl<P: Primitive> IntoIterator for CompoundPrimitive<P> {
    type Item = P;
    type IntoIter = <Vec<P> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<M: Material, P: Primitive> SkinnablePrimitive<M> for CompoundPrimitive<P> {
    type Target = SkinnedPrimitive<P, M>;

    fn apply_material(self, material: M) -> Self::Target {
        SkinnedPrimitive {
            material,
            primitives: self.0,
        }
    }
}

pub trait IntoPrimitive {
    type Primitive: Primitive;

    fn into_primitive(self) -> Self::Primitive;
}

impl<P: Primitive, IntoIter: IntoIterator<Item = P>> IntoPrimitive for IntoIter {
    type Primitive = CompoundPrimitive<P>;

    fn into_primitive(self) -> Self::Primitive {
        CompoundPrimitive(self.into_iter().collect())
    }
}
