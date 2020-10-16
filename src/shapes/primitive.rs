use super::{CompoundShape, HitResult, Shape, TransformablePrimitive, TransformableShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

#[derive(Debug)]
pub struct PrimitiveHitResult {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub front_face: bool,
    pub u: FloatType,
    pub v: FloatType,
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
            .filter_map(|shape| shape.intersect(&ray, t_min, t_max, stats))
            .min_by(|xr, yr| {
                xr.distance
                    .partial_cmp(&yr.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.iter
            .clone()
            .map(|a| a.bounding_box(t0, t1).clone())
            .my_fold_first(|a, b| BoundingBox::surrounding_box(&a, &b))
            .unwrap_or(BoundingBox::empty_box())
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
            .map(|hit_result| HitResult {
                distance: hit_result.distance,
                hit_point: hit_result.hit_point,
                surface_normal: hit_result.surface_normal,
                tangent: hit_result.tangent,
                bitangent: hit_result.bitangent,
                front_face: hit_result.front_face,
                material: &self.material,
                u: hit_result.u,
                v: hit_result.v,
            })
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
