use super::{CompoundShape, HitResult, Shape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

#[derive(Debug)]
pub struct UnskinnedHitResult {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub front_face: bool,
    pub u: FloatType,
    pub v: FloatType,
}

impl UnskinnedHitResult {
    pub fn into_hit_result<'a>(self, material: &'a dyn Material) -> HitResult<'a> {
        HitResult {
            distance: self.distance,
            hit_point: self.hit_point,
            surface_normal: self.surface_normal,
            tangent: self.tangent,
            bitangent: self.bitangent,
            front_face: self.front_face,
            u: self.u,
            v: self.v,
            material,
        }
    }
}

pub trait UnskinnedShape: Send + Sync + std::fmt::Debug {
    fn unskinned_intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<UnskinnedHitResult>;

    fn unskinned_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait CompoundUnskinnedShape: UnskinnedShape {
    type UnskinnedGeometryIterator: Iterator<Item = Box<dyn UnskinnedShape>>;

    fn into_unskinned_geometry_iterator(self) -> Self::UnskinnedGeometryIterator;
}

pub trait UnskinnedSimpleShape: UnskinnedShape {}

impl<S: UnskinnedSimpleShape + 'static> CompoundUnskinnedShape for S {
    type UnskinnedGeometryIterator = std::iter::Once<Box<dyn UnskinnedShape>>;

    fn into_unskinned_geometry_iterator(self) -> Self::UnskinnedGeometryIterator {
        let b: Box<dyn UnskinnedShape> = Box::new(self);
        std::iter::once(b)
    }
}

impl UnskinnedShape for Box<dyn UnskinnedShape> {
    fn unskinned_intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<UnskinnedHitResult> {
        self.as_ref().unskinned_intersect(ray, t_min, t_max, stats)
    }

    fn unskinned_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.as_ref().unskinned_bounding_box(t0, t1)
    }
}

#[derive(Clone, Debug)]
pub struct SkinnedShape<S: UnskinnedShape + Sized, M: Material> {
    shape: S,
    material: M,
}

impl<S: UnskinnedShape + Sized, M: Material> UnskinnedShape for SkinnedShape<S, M> {
    fn unskinned_intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<UnskinnedHitResult> {
        self.shape.unskinned_intersect(ray, t_min, t_max, stats)
    }

    fn unskinned_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.shape.unskinned_bounding_box(t0, t1)
    }
}

impl<S: CompoundUnskinnedShape, M: Material> CompoundUnskinnedShape for SkinnedShape<S, M> {
    type UnskinnedGeometryIterator = S::UnskinnedGeometryIterator;

    fn into_unskinned_geometry_iterator(self) -> Self::UnskinnedGeometryIterator {
        self.shape.into_unskinned_geometry_iterator()
    }
}

impl<S: UnskinnedShape, M: Material> Shape for SkinnedShape<S, M> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.unskinned_intersect(ray, t_min, t_max, stats)
            .map(|unskinned_result| unskinned_result.into_hit_result(&self.material))
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.unskinned_bounding_box(t0, t1)
    }
}

pub struct SkinnedShapeIterator<
    M: 'static + Material + Clone,
    Iter: Iterator<Item = Box<dyn UnskinnedShape>>,
> {
    material: M,
    iter: Iter,
}

impl<M: 'static + Material + Clone, Iter: Iterator<Item = Box<dyn UnskinnedShape>>> Iterator
    for SkinnedShapeIterator<M, Iter>
{
    type Item = Box<dyn Shape>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|child| {
            let b: Box<dyn Shape> = Box::new(SkinnedShape {
                material: self.material.clone(),
                shape: child,
            });
            b
        })
    }
}

impl<S: CompoundUnskinnedShape, M: 'static + Material + Clone> CompoundShape
    for SkinnedShape<S, M>
{
    type GeometryIterator = SkinnedShapeIterator<M, S::UnskinnedGeometryIterator>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        SkinnedShapeIterator {
            material: self.material,
            iter: self.shape.into_unskinned_geometry_iterator(),
        }
    }
}

pub trait SkinnableShape: UnskinnedShape + Sized {
    fn apply_material<M: Material>(self, material: M) -> SkinnedShape<Self, M>;
}

impl<S: UnskinnedShape> SkinnableShape for S {
    fn apply_material<M: Material>(self, material: M) -> SkinnedShape<Self, M> {
        SkinnedShape {
            shape: self,
            material,
        }
    }
}
